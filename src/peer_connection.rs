use crate::bittorrent_client::peer::Peer;
use crate::bittorrent_client::piece_queue::Piece;
use crate::bittorrent_client::piece_queue::PieceQueue;
use crate::constants::*;
use crate::errors::*;
use crate::p2p_messages::handshake::Handshake;
use crate::p2p_messages::interested::InterestedMessage;
use crate::p2p_messages::keep_alive::KeepAliveMessage;
use crate::p2p_messages::message_builder::MessageBuilder;
use crate::p2p_messages::message_builder::P2PMessage;
use crate::p2p_messages::message_trait::Message;
use crate::p2p_messages::piece::PieceMessage;
use crate::p2p_messages::request::RequestMessage;

use std::fs::File;
use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::RwLock;

/// # struct PeerConnection
/// Contains all information about the connection.
/// Fields:
///     - stream
///     - am_choked (1: choked, 0: unchoked)
///     - am_interested (1: interested, 0: not interested)
///     - downloaded_bytes
///     - total_bytes (size of file to download)
///     - piece_bytes (size of piece to download. First, this field is initialized using the
///       default value, but if we decide to download the last piece, this value is updated)
///     - pieces: Vector of pieces
///     - selected_piece: index of selected piece
///     - status 1: downloading  -> waiting piece message
///              0: not downloading -> we've received all requested blocks and can request the next one.
///     - piece: downloaded piece
#[derive(Debug)]
pub struct PeerConnection {
    torrent_name: String,
    download_dir_path: String,
    _hashes_list: Vec<u8>,
    stream: TcpStream,
    am_choked: u8,
    am_interested: u8,
    pieces: Vec<u8>,
    selected_piece: Piece,
    piece_status: bool,
    piece_queue: Arc<RwLock<PieceQueue>>,
    info_hash: Vec<u8>,
    our_peer_id: Vec<u8>,
    peer_id: Vec<u8>,
}

impl PeerConnection {
    /// Receives a client and a peer. Returns an initialized Peer connection
    /// In case the connection fails, returns error (CannotConnectToPeer)
    pub fn new(
        download_dir_path: String,
        torrent_name: String,
        hashes_list: Vec<u8>,
        peer: Peer,
        piece_queue: Arc<RwLock<PieceQueue>>,
        info_hash: Vec<u8>,
        our_peer_id: Vec<u8>,
    ) -> Result<PeerConnection, ClientError> {
        if let Ok(stream) = TcpStream::connect(format!("{}:{}", peer.ip(), peer.port())) {
            // Mejorar, ahora creamos una piece vacia para inicializar
            let p = Piece::new(0, 0, vec![]);

            let peer_conn = PeerConnection {
                download_dir_path,
                torrent_name,
                _hashes_list: hashes_list,
                stream,
                am_choked: CHOKED,
                am_interested: NOT_INTERESTED,
                pieces: vec![], // Los pieces que tiene el peer
                selected_piece: p,
                piece_status: true, // Si la selected_piece es valida o no
                piece_queue,
                info_hash,
                our_peer_id,
                peer_id: peer.id(),
            };
            return Ok(peer_conn);
        }
        Err(ClientError::CannotConnectToPeer)
    }

    /// Sends a handshake to a connected peer and tries to receive it from this one.
    /// On error, returns CannotConnectToPeer
    fn exchange_handshake(
        &mut self,
        peer_id: Vec<u8>,
        info_hash: Vec<u8>,
        our_peer_id: Vec<u8>,
    ) -> Result<(), ClientError> {
        let handshake =
            Handshake::new_from_param("BitTorrent protocol", info_hash.clone(), our_peer_id);
        if let Ok(()) = handshake.send_msg(&mut self.stream) {
            let handshake_res =
                Handshake::read_msg(&mut self.stream).map_err(ClientError::MessageReadingError)?;
            if handshake_res.is_valid(info_hash, peer_id) {
                return Ok(());
            }
        }
        Err(ClientError::CannotConnectToPeer)
    }

    /// Makes the exchange of messages following the BitTorrent protocol.
    /// Finally, on success, it returns the downloaded piece.
    /// Otherwise, it returns an error.
    ///
    /// -> Note that if the other peer chokes us, the message exchange will end, otherwise,
    /// it will continue until we download the piece or some error arises.
    pub fn start_download(&mut self) -> Result<(), ClientError> {
        self.exchange_handshake(
            self.get_peer_id(),
            self.get_info_hash(),
            self.get_our_peer_id(),
        )?;

        self.piece_status = self.fetch_piece();
        loop {
            // Mientras no hayamos bajado toda la pieza seleccionada
            while self.selected_piece.get_dl() < self.selected_piece.get_tl() && self.piece_status {
                if self.am_choked == UNCHOKED
                    && self.am_interested == INTERESTED
                    && self.selected_piece.get_rq() < self.selected_piece.get_tl()
                {
                    self.request_a_block()?;
                }

                self.keep_connection_alive();

                let msg = MessageBuilder::build(&mut self.stream)
                    .map_err(ClientError::MessageReadingError)?;

                // Si estamos explicitamente choked
                if let P2PMessage::Choke(_msg) = msg {
                    println!("The peer choked us");
                    self.am_choked = CHOKED;
                    // Borrar la data
                    self.return_piece();
                    self.piece_status = self.fetch_piece();
                } else {
                    self.handle_msg(msg);
                }

                // Si nos mandaron bitfield chequemos que el peer tenga nuestra selected_piece
                // si no la tiene, pedimos otra hasta tener un match
                while !self.pieces.is_empty() && !self.has_piece(self.selected_piece.get_idx()) {
                    // Borrar la data
                    self.return_piece();
                    self.piece_status = self.fetch_piece();
                }

                // Si aun no estamos interesados, mandamos interested al peer una vez que
                // chequeamos que tiene la piece que queremos
                if self.am_interested == NOT_INTERESTED {
                    self.interested_in_piece()?;
                }
            }

            // Chequemos que la queue nos devolvio una pieza valida
            if !self.piece_status {
                println!("The queue served us an invalid piece");
                return Err(ClientError::ProtocolError);
            };

            // Si la pieza es valida intentamos escribir a archivo
            if self.selected_piece.piece_is_valid() {
                let e = self.store_piece_in_file();

                // Si no se pudo borramos data, devolvemos pieza a queue y retornamos err
                if e.is_err() {
                    // Borrar la data
                    println!("Cannot write piece to file");
                    self.return_piece();
                    return e;
                }
                // Si se escribio a archivo pedimos otra pieza
                self.piece_status = self.fetch_piece();
            } else {
                println!("The downloaded piece is not valid");
            }
        }
    }

    /// According to the received message, it makes some decission.
    /// Bitfield -> initializes peer's piece vector
    /// Have -> updates peer's piece vector
    /// Unchoke -> sets am_choked = 0
    /// Piece -> handle piece msg
    fn handle_msg(&mut self, message: P2PMessage) {
        match message {
            P2PMessage::Bitfield(msg) => self.pieces = msg.get_pieces(),
            P2PMessage::Have(msg) => self.update_pieces(msg.get_piece_index()),
            P2PMessage::Unchoke(_msg) => self.am_choked = UNCHOKED,
            P2PMessage::Piece(msg) => self.handle_piece(msg),
            _ => (),
        }
    }

    /// Sets status as NOT_DOWNLOADING (0), the checks if the received block is valid.
    /// Finally, updates the value of the downloaded byte and appends the received block to self.piece
    fn handle_piece(&mut self, msg: PieceMessage) {
        if (msg.get_begin() == self.selected_piece.get_dl())
            && (msg.get_piece_index() == self.selected_piece.get_idx())
        {
            let block = msg.get_block();
            self.selected_piece.add_to_dl(block.len() as u32);
            self.display_progress_bar();
            self.selected_piece.add_block(block);
        }
    }

    /// Receives a message and tries to send it to the connected peer.
    /// Tries to send it 10 times. If all sendings fail, returns an error.
    fn send_message<T: Message>(&mut self, msg: T) -> Result<(), ClientError> {
        if let Ok(()) = msg.send_msg(&mut self.stream) {
            Ok(())
        } else {
            Err(ClientError::ProtocolError)
        }
    }

    /// Writes bytes of the downloaded piece in a file.
    fn store_piece_in_file(&mut self) -> Result<(), ClientError> {
        let path = format!(
            "{}/{}_piece_{}",
            self.download_dir_path,
            self.torrent_name,
            self.selected_piece.get_idx(),
        );
        if let Ok(mut file) = File::create(path) {
            if file.write_all(&self.selected_piece.get_data()).is_ok() {
                return Ok(());
            }
        }
        println!("ERROR: Cannot store the piece in the file :(");
        Err(ClientError::StoringPieceError)
    }

    /// Sends a KeepAlive message
    fn keep_connection_alive(&mut self) {
        let keep_alive_msg = KeepAliveMessage::new();
        if self.send_message(keep_alive_msg).is_ok() {}
    }

    /// Sends a Request message with the current block and sets status = DOWNLOADING (1)
    /// If sending failes, returns an error.
    fn request_a_block(&mut self) -> Result<(), ClientError> {
        let piece_idx = self.selected_piece.get_idx();

        // If we have unread requests skip doing more requests
        if self.selected_piece.get_rq() > self.selected_piece.get_dl() {
            return Ok(());
        };

        // We make 10 requests
        for _ in 0..10 {
            // If we already requested the whole piece, stop requesting
            if self.selected_piece.get_rq() >= self.selected_piece.get_tl() {
                break;
            };
            let begin = self.selected_piece.get_rq();
            let block_length = self.selected_piece.next_block_length();
            if let Ok(request_msg) = RequestMessage::new(piece_idx, begin, block_length) {
                self.send_message(request_msg)?;
            }
            self.selected_piece.add_to_rq(block_length)
        }

        Ok(())
    }

    /// Sends Interested message and sets am_interested = INTERESTED (1)
    fn interested_in_piece(&mut self) -> Result<(), ClientError> {
        let interested_msg = InterestedMessage::new();
        self.am_interested = INTERESTED;
        self.send_message(interested_msg)
    }

    pub fn get_peer_id(&self) -> Vec<u8> {
        self.peer_id.clone()
    }

    pub fn get_our_peer_id(&self) -> Vec<u8> {
        self.our_peer_id.clone()
    }

    pub fn get_info_hash(&self) -> Vec<u8> {
        self.info_hash.clone()
    }

    /// Checks if the peer has any piece to download.
    /// For this, we check if any byte is different from zero
    fn _has_any_piece(&self) -> bool {
        for i in &self.pieces {
            if *i != 0 {
                return true;
            }
        }
        false
    }

    //Checks if the peer has a particular piece to download
    //We check if the piece index is different from zero
    fn has_piece(&self, index: u32) -> bool {
        let u8_index = index as usize;
        if self.pieces[u8_index] != 0 {
            return true;
        }
        false
    }

    // Pedir lock y pedir pieza a la queue, sacar del option y devolver true si habia pieza
    // false si no
    fn fetch_piece(&mut self) -> bool {
        let mut pq_lock = self.piece_queue.write().unwrap();

        if let Some(option_piece) = pq_lock.get_next_piece() {
            self.selected_piece = option_piece;
            return true;
        }
        false
    }

    fn return_piece(&mut self) {
        let mut pq_lock = self.piece_queue.write().unwrap();
        pq_lock.put_in_queue(
            self.selected_piece.get_idx(),
            self.selected_piece.get_tl(),
            self.selected_piece.get_hash(),
        );
    }

    fn _queue_is_empty(&mut self) -> bool {
        let pq_lock = self.piece_queue.read().unwrap();
        pq_lock.is_empty()
    }

    /// Receives a piece index and mark this one with a 1 in the peer's vector pieces
    fn update_pieces(&mut self, piece_idx: u32) {
        let n_shift = 7 - (piece_idx % 8);
        let mask: u8 = 1 << n_shift;
        let idx: usize = (piece_idx / 8) as usize;
        self.pieces[idx] |= mask;
    }

    /// Prints progress bar
    fn display_progress_bar(&self) {
        let percent: u32 = 100 * self.selected_piece.get_dl() / self.selected_piece.get_tl();
        // let bar = "█".repeat((percent / 10) as usize);

        // if percent < 100 {
        //     println!("DOWNLOADING... PIECE_N {} {} {}%\n",  self.selected_piece.get_idx(), bar, percent);
        // } else {
        //     println!("DOWNLOADED PIECE_N {}\n", self.selected_piece.get_idx());
        // }
        if percent >= 100 {
            println!("DOWNLOADED PIECE_N {}\n", self.selected_piece.get_idx());
        }
    }
}
