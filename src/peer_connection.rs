use crate::constants::*;
use crate::errors::*;

use crate::bittorrent_client::client::Client;
use crate::bittorrent_client::peer::Peer;

use crate::p2p_messages::handshake::Handshake;
use crate::p2p_messages::interested::InterestedMessage;
use crate::p2p_messages::keep_alive::KeepAliveMessage;
use crate::p2p_messages::message_builder::MessageBuilder;
use crate::p2p_messages::message_builder::P2PMessage;
use crate::p2p_messages::message_trait::Message;
use crate::p2p_messages::piece::PieceMessage;
use crate::p2p_messages::request::RequestMessage;

use std::net::TcpStream;

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
    am_choked: u8,
    am_interested: u8,
    downloaded_bytes: u32,
    total_bytes: u32,
    piece_bytes: u32,
    pieces: Vec<u8>,
    selected_piece: Option<u32>,
    status: u8,
    piece: Vec<u8>,
}

impl PeerConnection {
    /// Receives a client and a peer. Returns an initialized Peer connection
    /// In case the connection fails, returns error (CannotConnectToPeer)
    pub fn new(client: &Client, peer: &Peer) -> Result<PeerConnection, ClientError> {
        if let Ok(stream) = TcpStream::connect(format!("{}:{}", peer.ip(), peer.port())) {
            let torrent_info = client.get_torrent_info();

            let mut peer_conn = PeerConnection {
                stream,
                am_choked: CHOKED,
                am_interested: NOT_INTERESTED,
                downloaded_bytes: 0,
                total_bytes: torrent_info.get_length(),
                piece_bytes: torrent_info.get_piece_length(),
                pieces: vec![
                    0;
                    (torrent_info.get_length() / torrent_info.get_piece_length()) as usize
                ],
                selected_piece: None,
                status: NOT_DOWNLOADING,
                piece: vec![],
            };

            if let Ok(()) = peer_conn.exchange_handshake(client, peer) {
                return Ok(peer_conn);
            }
        }
        Err(ClientError::CannotConnectToPeer)
    }

    /// Sends a handshake to a connected peer and tries to receive it from this one.
    /// On error, returns CannotConnectToPeer
    fn exchange_handshake(&mut self, client: &Client, peer: &Peer) -> Result<(), ClientError> {
        let handshake = Handshake::new(client, "BitTorrent protocol");
        if let Ok(()) = handshake.send_msg(&mut self.stream) {
            let handshake_res =
                Handshake::read_msg(&mut self.stream).map_err(ClientError::MessageReadingError)?;
            if handshake_res.is_valid(client.get_torrent_info().get_info_hash(), peer.id()) {
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
    pub fn download_piece(&mut self) -> Result<(u32, Vec<u8>), ClientError> {
        println!("Downloading has been started\n");
        while self.downloaded_bytes < self.piece_bytes {
            self.keep_connection_alive();

            let msg = MessageBuilder::build(&mut self.stream)
                .map_err(ClientError::MessageReadingError)?;
            if let P2PMessage::Choke(_) = msg {
                self.am_choked = CHOKED;
                return Err(ClientError::ProtocolError);
            }
            self.handle_msg(msg);

            if self.am_interested == NOT_INTERESTED && self.has_any_piece() {
                self.select_piece();
                self.interested_in_piece()?;
            }

            if self.am_choked == UNCHOKED
                && self.am_interested == INTERESTED
                && self.status == NOT_DOWNLOADING
            {
                self.request_a_block()?;
            }
        }
        Ok((
            self.selected_piece.expect("This shouldn't be possible"),
            self.piece.clone(),
        ))
    }

    /// Receives a message and tries to send it to the connected peer.
    /// Tries to send it 10 times. If all sendings fail, returns an error.
    fn send_message<T: Message>(&mut self, msg: T) -> Result<(), ClientError> {
        loop {
            let mut _i = 0;
            if let Ok(()) = msg.send_msg(&mut self.stream) {
                return Ok(());
            } else if _i == 10 {
                return Err(ClientError::ProtocolError);
            }
            _i += 1;
        }
    }

    /// Sends a KeepAlive message
    fn keep_connection_alive(&mut self) {
        let keep_alive_msg = KeepAliveMessage::new();
        if self.send_message(keep_alive_msg).is_ok() {}
    }

    /// Sends a Request message with the current block and sets status = DOWNLOADING (1)
    /// If sending failes, returns an error.
    fn request_a_block(&mut self) -> Result<(), ClientError> {
        if let Some(piece_idx) = self.selected_piece {
            let begin = self.downloaded_bytes;
            let block_length = self.next_block_length();

            if let Ok(request_msg) = RequestMessage::new(piece_idx, begin, block_length) {
                self.send_message(request_msg)?;
            }
            self.status = DOWNLOADING;
            return Ok(());
        }
        Err(ClientError::ProtocolError)
    }

    /// Sends Interested message and sets am_interested = INTERESTED (1)
    fn interested_in_piece(&mut self) -> Result<(), ClientError> {
        let interested_msg = InterestedMessage::new();
        self.am_interested = INTERESTED;
        self.send_message(interested_msg)
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
        self.status = NOT_DOWNLOADING;
        if let Some(idx) = self.selected_piece {
            if (msg.get_begin() == self.downloaded_bytes) && (msg.get_piece_index() == idx) {
                let mut block = msg.get_block();
                self.downloaded_bytes += block.len() as u32;
                self.display_progress_bar();
                self.piece.append(&mut block);
            }
        }
    }

    /// Checks if the peer has any piece to download.
    /// For this, we check if any byte is different from zero
    fn has_any_piece(&self) -> bool {
        for i in &self.pieces {
            if *i != 0 {
                return true;
            }
        }
        false
    }

    /// Iterates over peer's pieces vector and finds the first available piece
    fn select_piece(&mut self) {
        for i in 0..self.pieces.len() {
            for j in 0..8 {
                let mask: u8 = 1 << (7 - j);
                if (self.pieces[i] & mask) != 0 {
                    let selected_piece = (8 * i + j) as u32;
                    self.selected_piece = Some(selected_piece);
                    if self.is_last_piece(selected_piece) {
                        self.piece_bytes = self.total_bytes % self.piece_bytes;
                    }
                    return;
                }
            }
        }
    }

    fn is_last_piece(&self, piece_idx: u32) -> bool {
        piece_idx == ((self.total_bytes as f32 / self.piece_bytes as f32).ceil() as u32 - 1)
    }

    /// Receives a piece index and mark this one with a 1 in the peer's vector pieces
    fn update_pieces(&mut self, piece_idx: u32) {
        let n_shift = 7 - (piece_idx % 8);
        let mask: u8 = 1 << n_shift;
        let idx: usize = (piece_idx / 8) as usize;
        self.pieces[idx] |= mask;
    }

    /// Return the size of the next block
    pub fn next_block_length(&self) -> u32 {
        let block_length = 1 << 14;
        let left = self.piece_bytes - self.downloaded_bytes;
        if left < block_length {
            return left;
        }

        block_length
    }

    /// Prints progress bar
    fn display_progress_bar(&self) {
        let percent: u32 = 100 * self.downloaded_bytes / self.piece_bytes;
        let bar = "█".repeat((percent / 10) as usize);

        if percent < 100 {
            print!("{esc}c", esc = 27 as char);
            println!("DOWNLOADING... {} {}%\n", bar, percent);
        } else {
            print!("{esc}c", esc = 27 as char);
            println!("DOWNLOADED :D  {} {}%\n", bar, percent);
        }
    }
}
