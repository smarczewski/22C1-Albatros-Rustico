use crate::bitfield::PieceBitfield;
use crate::bittorrent_client::client::Client;
use crate::bittorrent_client::peer::Peer;
use crate::bittorrent_client::piece_queue::Piece;
use crate::bittorrent_client::piece_queue::PieceQueue;
use crate::errors::*;
use crate::event_messages::*;
use crate::p2p_messages::handshake::Handshake;
use crate::p2p_messages::interested::InterestedMessage;
use crate::p2p_messages::keep_alive::KeepAliveMessage;
use crate::p2p_messages::message_builder::MessageBuilder;
use crate::p2p_messages::message_builder::P2PMessage;
use crate::p2p_messages::message_trait::Message;
use crate::p2p_messages::piece::PieceMessage;
use crate::p2p_messages::request::RequestMessage;

use std::fs::{self, File};
use std::io::Write;
use std::net::TcpStream;
use std::path::Path;
use std::sync::{mpsc::Sender, Arc, RwLock};
use std::thread;
use std::time::Duration;
use std::vec;

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
    stream: TcpStream,
    client: Client,
    peer: Peer,
    am_choked: bool,
    am_interested: bool,
    pieces: PieceBitfield,
    piece_queue: Arc<RwLock<PieceQueue>>,
    tx_client: Sender<NewEvent>,
}

impl PeerConnection {
    /// Receives a client and a peer. Returns an initialized Peer connection
    /// In case the connection fails, returns error (CannotConnectToPeer)
    pub fn new(
        client: Client,
        peer: Peer,
        piece_queue: Arc<RwLock<PieceQueue>>,
        tx_client: Sender<NewEvent>,
    ) -> Result<PeerConnection, ClientError> {
        if let Ok(stream) = peer.connect() {
            let number_of_pieces = client.get_torrent_info().get_n_pieces();
            let bitfield = vec![0; (number_of_pieces as f32 / 8.0).ceil() as usize];

            if stream.set_read_timeout(Some(Duration::new(5, 0))).is_ok()
                && tx_client
                    .send(NewEvent::NewConnection(peer.clone()))
                    .is_ok()
            {
                return Ok(PeerConnection {
                    stream,
                    client,
                    peer,
                    am_choked: true,
                    am_interested: false,
                    pieces: PieceBitfield::new_from_vec(bitfield, number_of_pieces),
                    piece_queue,
                    tx_client,
                });
            }
        }
        Err(ClientError::CannotConnectToPeer)
    }

    /// Sends a handshake to a connected peer and tries to receive it from this one.
    /// On error, returns CannotConnectToPeer
    fn exchange_handshake(&mut self) -> Result<(), DownloadError> {
        let handshake = Handshake::new(&self.client, "BitTorrent protocol");
        if handshake.send_msg(&mut self.stream).is_ok() {
            if let Ok(handshake_res) = Handshake::read_msg(&mut self.stream) {
                if handshake_res.is_valid(self.client.get_torrent_info().get_info_hash()) {
                    return Ok(());
                }
            }
        }
        Err(DownloadError::HandshakeError)
    }

    pub fn start_download(
        &mut self,
        bf_pieces: Arc<RwLock<PieceBitfield>>,
        dl_finished: Arc<RwLock<bool>>,
    ) {
        if self.exchange_handshake().is_err() {
            return self.drop_connection(None);
        }

        loop {
            if let Ok(mut piece) = self.fetch_piece() {
                match self.download_piece(&mut piece) {
                    Ok(_) => self.handle_new_piece(piece),

                    Err(DownloadError::InvalidPiece) => self.return_piece(piece),

                    Err(DownloadError::CannotReadPeerMessage) => {
                        println!("Error leer msj");
                        return self.drop_connection(Some(piece));
                    }
                    Err(DownloadError::PeerChokedUs) => {
                        return self.drop_connection(Some(piece));
                    }
                    _ => (),
                }
                continue;
            } else if *dl_finished.read().unwrap() || !self.has_any_wanted_piece(&bf_pieces) {
                return self.drop_connection(None);
            } else {
                thread::yield_now();
            }
        }
    }

    fn has_any_wanted_piece(&self, dl_pieces: &Arc<RwLock<PieceBitfield>>) -> bool {
        let wanted_pieces = dl_pieces.read().unwrap().get_complement();
        self.pieces.there_is_match(&wanted_pieces)
    }

    fn handle_new_piece(&mut self, mut piece: Piece) {
        if self.store_piece_in_file(&mut piece).is_ok() {
            println!(
                "DOWNLOADED PIECE_N {} - from peer: {:?}\n",
                piece.get_idx(),
                self.peer.id()
            );
            let _ = self.tx_client.send(NewEvent::NewDownloadedPiece(piece));
        } else {
            self.return_piece(piece);
        }
    }

    fn drop_connection(&mut self, curr_piece: Option<Piece>) {
        if let Some(piece) = curr_piece {
            println!("retorna pieza");
            self.return_piece(piece);
        }
        let _ = self.tx_client.send(NewEvent::ConnectionDropped);
    }

    /// Makes the exchange of messages following the BitTorrent protocol.
    /// Finally, on success, it returns the downloaded piece.
    /// Otherwise, it returns an error.
    ///
    /// -> Note that if the other peer chokes us, the message exchange will end, otherwise,
    /// it will continue until we download the piece or some error arises.
    pub fn download_piece(&mut self, piece: &mut Piece) -> Result<(), DownloadError> {
        // if !self.pieces.has_piece(piece.get_idx()){
        //     return Err(DownloadError::InvalidPiece);
        // }

        while piece.get_dl() < piece.get_tl() {
            self.keep_connection_alive();

            if !self.am_interested {
                self.interested_in_piece();
            }

            if !self.am_choked && self.am_interested {
                self.request_a_piece(piece);
            }

            self.receive_message(piece)?;
        }

        if piece.piece_is_valid() {
            return Ok(());
        }
        Err(DownloadError::InvalidPiece)
    }

    fn receive_message(&mut self, piece: &mut Piece) -> Result<(), DownloadError> {
        if let Ok(msg) = MessageBuilder::build(&mut self.stream) {
            if let P2PMessage::Choke(_) = msg {
                self.am_choked = true;
                return Err(DownloadError::PeerChokedUs);
            }
            self.handle_msg(msg, piece);
            return Ok(());
        }
        Err(DownloadError::CannotReadPeerMessage)
    }

    /// According to the received message, it makes some decission.
    /// Bitfield -> initializes peer's piece vector
    /// Have -> updates peer's piece vector
    /// Unchoke -> sets am_choked = 0
    /// Piece -> handle piece msg
    fn handle_msg(&mut self, message: P2PMessage, piece: &mut Piece) {
        match message {
            P2PMessage::Bitfield(msg) => self.pieces.add_multiple_pieces(msg.get_pieces()),
            P2PMessage::Have(msg) => self.pieces.add_a_piece(msg.get_piece_index()),
            P2PMessage::Unchoke(_msg) => self.am_choked = false,
            P2PMessage::Piece(msg) => self.handle_piece_msg(msg, piece),
            _ => (),
        }
    }

    /// Sets status as NOT_DOWNLOADING (0), the checks if the received block is valid.
    /// Finally, updates the value of the downloaded byte and appends the received block to self.piece
    fn handle_piece_msg(&mut self, msg: PieceMessage, piece: &mut Piece) {
        if (msg.get_begin() == piece.get_dl()) && (msg.get_piece_index() == piece.get_idx()) {
            let block = msg.get_block();
            piece.add_to_dl(block.len() as u32);
            piece.add_block(block);
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
    fn store_piece_in_file(&self, piece: &mut Piece) -> Result<(), ()> {
        let download_dir_path = self.client.get_download_dir();
        let torrent_name = self.client.get_torrent_info().get_name();

        if !Path::new(&download_dir_path).exists()
            && fs::create_dir_all(&download_dir_path).is_err()
        {
            return Err(());
        }

        let path = format!(
            "{}/{}_piece_{}",
            download_dir_path,
            torrent_name,
            piece.get_idx(),
        );
        if let Ok(mut file) = File::create(path) {
            if file.write_all(&piece.get_data()).is_ok() {
                return Ok(());
            }
        }
        Err(())
    }

    /// Sends a KeepAlive message
    fn keep_connection_alive(&mut self) {
        let keep_alive_msg = KeepAliveMessage::new();
        let _ = self.send_message(keep_alive_msg);
    }

    /// Sends a Request message with the current block and sets status = DOWNLOADING (1)
    /// If sending failes, returns an error.
    fn request_a_piece(&mut self, piece: &mut Piece) {
        while piece.get_rq() < piece.get_tl() {
            let begin = piece.get_rq();
            let block_length = piece.next_block_length();

            if let Ok(request_msg) = RequestMessage::new(piece.get_idx(), begin, block_length) {
                if request_msg.send_msg(&mut self.stream).is_ok() {
                    piece.add_to_rq(block_length);
                }
            }
        }
    }

    /// Sends Interested message and sets am_interested = INTERESTED (1)
    fn interested_in_piece(&mut self) {
        let interested_msg = InterestedMessage::new();
        if self.send_message(interested_msg).is_ok() {
            self.am_interested = true;
        }
    }

    // Pedir lock y pedir pieza a la queue, sacar del option y devolver true si habia pieza
    // false si no
    fn fetch_piece(&mut self) -> Result<Piece, ()> {
        if let Ok(mut pq_lock) = self.piece_queue.write() {
            if let Some(option_piece) = pq_lock.get_next_piece() {
                return Ok(option_piece);
            }
        }

        Err(())
    }

    fn return_piece(&mut self, mut piece: Piece) {
        piece.reset_info();
        let mut pq_lock = self.piece_queue.write().unwrap();
        pq_lock.push_back(piece);
    }
}
