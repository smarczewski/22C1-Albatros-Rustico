use std::fs::DirEntry;
use std::io::Seek;
use std::net::TcpStream;
use std::sync::mpsc::Sender;

use crate::bitfield::PieceBitfield;
use crate::constants::*;
use crate::encoding_decoding::encoder::Encoder;
use crate::errors::ServerError;
use crate::logging::msg_coder::MsgCoder;
use crate::p2p_messages::bitfield::BitfieldMsg;
use crate::p2p_messages::handshake::Handshake;
use crate::p2p_messages::message_builder::MessageBuilder;
use crate::p2p_messages::message_builder::P2PMessage;
use crate::p2p_messages::message_trait::Message;
use crate::p2p_messages::piece::PieceMsg;
use crate::p2p_messages::request::RequestMsg;
use crate::p2p_messages::unchoke::UnchokeMsg;
use crate::piece::Piece;
use crate::torrent_info::TorrentInfo;

use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::SeekFrom;
use std::sync::{Arc, RwLock};
use std::time::Duration;

/// # struct PeerConnection (server)
/// Contains all information about the connection.
/// Fields:
///     - stream
///     - peer_id
///     - is_choked -> peer is choked
///     - is_interested -> peer is interested
///     - our_pieces -> PieceBitfield of our pieces
///     - torrent_info -> torrent that peer is interested in
///     - download_path
///     - piece -> piece requested by the peer.
///     - tx_logger
pub struct PeerConnection {
    stream: TcpStream,
    peer_id: Vec<u8>,
    is_choked: u8,
    is_interested: u8,
    our_pieces: Arc<RwLock<PieceBitfield>>,
    torrent_info: TorrentInfo,
    download_path: String,
    piece: Option<Piece>,
    tx_logger: Sender<String>,
}

impl PeerConnection {
    /// Receives a handshake, then sends a handshake to the peer.
    /// Also, it initializes the peer connection using the information
    /// of the torrent the peer requested.
    pub fn new(
        mut stream: TcpStream,
        torrents: Vec<(TorrentInfo, Arc<RwLock<PieceBitfield>>)>,
        download_path: String,
        tx_logger: Sender<String>,
    ) -> Result<PeerConnection, ServerError> {
        if let Ok(handshake) = Handshake::read_msg(&mut stream) {
            if stream
                .set_read_timeout(Some(Duration::new(TWO_MINUTES, 0)))
                .is_ok()
            {
                let info_hash = handshake.get_info_hash();
                let peer_id = handshake.get_peer_id();
                let (torrent_info, our_pieces) = get_torrent_info(&info_hash, torrents)?;
                PeerConnection::send_handshake(info_hash, &mut stream)?;

                let peer_conn = PeerConnection {
                    stream,
                    peer_id,
                    is_choked: CHOKED,
                    is_interested: NOT_INTERESTED,
                    our_pieces,
                    torrent_info,
                    download_path,
                    piece: None,
                    tx_logger,
                };
                peer_conn.announce_new_connection();
                return Ok(peer_conn);
            }
        }
        Err(ServerError::HandshakeError)
    }

    fn send_handshake(info_hash: Vec<u8>, stream: &mut TcpStream) -> Result<(), ServerError> {
        let our_handshake = Handshake::new_from_param(
            "BitTorrent protocol",
            info_hash,
            CLIENT_ID.as_bytes().to_vec(),
        );
        match our_handshake.send_msg(stream) {
            Ok(_) => Ok(()),
            Err(_) => Err(ServerError::HandshakeError),
        }
    }

    /// Sends the Bitfield message as the first message, then it listening for new messages
    /// When a new message from the other peer arrives, it is handled.
    pub fn handle_connection(&mut self) {
        if let Ok(pieces) = self.our_pieces.read() {
            if let Ok(bf_msg) = BitfieldMsg::new(pieces.get_vec()) {
                let _ = bf_msg.send_msg(&mut self.stream);
            }
        }

        while let Ok(msg) = MessageBuilder::build(&mut self.stream) {
            self.handle_msg(msg);
        }
    }

    /// According to the received message, it makes some decission.
    fn handle_msg(&mut self, message: P2PMessage) {
        match message {
            P2PMessage::Interested(_msg) => self.handle_interested_msg(),
            P2PMessage::NotInterested(_msg) => self.is_interested = NOT_INTERESTED,
            P2PMessage::Request(msg) => self.handle_request(msg),
            _ => (),
        }
    }

    fn handle_interested_msg(&mut self) {
        self.is_interested = INTERESTED;
        if UnchokeMsg::new().send_msg(&mut self.stream).is_ok() {
            self.is_choked = UNCHOKED;
        }
    }

    /// Receives a Request Message, loads the piece in self.piece
    /// and then gets the correct block of bytes from this piece.
    /// The whole piece is loaded because the peer probably keeps requesting blocks
    /// of the same piece, so by doing this we avoid reading the same piece many times.
    fn handle_request(&mut self, msg: RequestMsg) {
        let piece_idx = msg.get_piece_index();
        if self.is_interested == NOT_INTERESTED
            || self.is_choked == CHOKED
            || !self.have_the_piece(piece_idx)
        {
            return;
        }

        if let Some(piece) = &self.piece {
            if piece.get_idx() != piece_idx {
                match self.load_piece(piece_idx) {
                    Ok(piece) => self.piece = piece,
                    Err(_) => return,
                }
            }
        } else if self.piece.is_none() {
            match self.load_piece(piece_idx) {
                Ok(piece) => self.piece = piece,
                Err(_) => return,
            }
        }

        let block = self.get_block(msg.get_begin(), msg.get_block_length());
        if let Ok(msg) = PieceMsg::new(piece_idx, msg.get_begin(), block) {
            if msg.send_msg(&mut self.stream).is_ok() {
                self.announce_piece_served(msg);
            }
        }
    }

    fn have_the_piece(&self, piece_idx: u32) -> bool {
        if let Ok(pieces) = self.our_pieces.read() {
            return pieces.has_piece(piece_idx);
        }
        false
    }

    fn load_piece(&self, piece_idx: u32) -> Result<Option<Piece>, ServerError> {
        if let Ok(files) = fs::read_dir(&self.download_path) {
            for file in files.flatten() {
                let file_name = file.file_name().to_string_lossy().to_string();
                let piece_name = format!("{}_piece_{}", self.torrent_info.get_name(), piece_idx);

                // Get piece from directory
                if file_name == piece_name {
                    if let Ok(loaded_piece) = self.piece_from_dir(&file, piece_idx) {
                        return Ok(loaded_piece);
                    }
                }
                // Get piece from a entire downloaded file
                else if file_name == self.torrent_info.get_name() {
                    if let Ok(loaded_piece) = self.piece_from_file(&file, piece_idx) {
                        return Ok(loaded_piece);
                    }
                }
            }
        }
        Err(ServerError::NoSuchDirectory)
    }

    fn piece_from_dir(&self, file: &DirEntry, idx: u32) -> Result<Option<Piece>, ServerError> {
        if let Ok(mut piece_file) = File::open(file.path()) {
            let mut piece = Piece::new(idx, 0, vec![0u8; 20]);
            let mut buffer = Vec::new();
            if piece_file.read_to_end(&mut buffer).is_ok() {
                piece.add_block(buffer);
                return Ok(Some(piece));
            }
        }
        Err(ServerError::NoSuchDirectory)
    }

    fn piece_from_file(&self, file: &DirEntry, idx: u32) -> Result<Option<Piece>, ServerError> {
        if let Ok(mut downloaded_file) = File::open(file.path()) {
            let pos = idx * self.torrent_info.get_piece_length();
            if downloaded_file.seek(SeekFrom::Start(pos as u64)).is_ok() {
                let mut buffer = vec![0u8; self.torrent_info.length_of_piece_n(idx) as usize];
                if downloaded_file.read_exact(&mut buffer).is_ok() {
                    let mut piece = Piece::new(idx, 0, vec![0u8; 20]);
                    piece.add_block(buffer);
                    return Ok(Some(piece));
                }
            }
        }
        Err(ServerError::NoSuchDirectory)
    }

    fn get_block(&self, begin: u32, block_length: u32) -> Vec<u8> {
        let mut block = Vec::new();
        if let Some(piece) = &self.piece {
            let piece_data = piece.get_data();
            for i in begin..begin + block_length {
                block.push(piece_data[i as usize]);
            }
        }
        block
    }

    fn announce_new_connection(&self) {
        if self
            .tx_logger
            .send(MsgCoder::generate_message(
                START_LOG_TYPE,
                SERVER_MODE_LOG,
                format!(
                    "Torrent: {} - Peer: {} connect to us\n",
                    self.torrent_info.get_name(),
                    Encoder.urlencode(&self.peer_id)
                ),
            ))
            .is_err()
        {
            println!("Failed to log new connection");
        }
    }

    fn announce_piece_served(&self, msg: PieceMsg) {
        let piece_idx = msg.get_piece_index();
        let begin = msg.get_begin();
        let block = msg.get_block().len();

        if self
            .tx_logger
            .send(MsgCoder::generate_message(
                GENERIC_LOG_TYPE,
                SERVER_MODE_LOG,
                format!(
                    "Torrent: {} - Serving piece: {}, begin: {} and block len: {} to {}\n",
                    self.torrent_info.get_name(),
                    piece_idx,
                    begin,
                    block,
                    Encoder.urlencode(&self.peer_id)
                ),
            ))
            .is_err()
        {
            println!("Failed to log new connection");
        }
    }
}

fn get_torrent_info(
    info_hash: &[u8],
    torrents: Vec<(TorrentInfo, Arc<RwLock<PieceBitfield>>)>,
) -> Result<(TorrentInfo, Arc<RwLock<PieceBitfield>>), ServerError> {
    for torrent in torrents {
        if torrent.0.get_info_hash() == *info_hash {
            return Ok((torrent.0, torrent.1));
        }
    }
    Err(ServerError::CannotFindTorrent)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::torrent_info::TorrentInfo;
    use sha1::{Digest, Sha1};

    fn load_piece_for_unit_test(piece_idx: u32) -> Result<Option<Piece>, ServerError> {
        let torrent_name = "ubuntu-20.04.4-desktop-amd64.iso";
        let download_dir = "files_for_testing/downloaded_files";
        let length = 262144;

        if let Ok(files) = fs::read_dir(download_dir) {
            for file in files.flatten() {
                let file_name = file.file_name().to_string_lossy().to_string();
                let piece_name = format!("{}_piece_{}", torrent_name, piece_idx);

                if file_name == piece_name {
                    if let Ok(loaded_piece) = piece_from_dir_unit_test(&file, piece_idx) {
                        return Ok(loaded_piece);
                    }
                } else if file_name == torrent_name {
                    if let Ok(loaded_piece) = piece_from_file_unit_test(&file, piece_idx, length) {
                        return Ok(loaded_piece);
                    }
                }
            }
        }
        Err(ServerError::NoSuchDirectory)
    }

    fn piece_from_dir_unit_test(file: &DirEntry, idx: u32) -> Result<Option<Piece>, ServerError> {
        if let Ok(mut piece_file) = File::open(file.path()) {
            let mut piece = Piece::new(idx, 0, vec![0u8; 20]);
            let mut buffer = Vec::new();
            if piece_file.read_to_end(&mut buffer).is_ok() {
                piece.add_block(buffer);
                return Ok(Some(piece));
            }
        }
        Err(ServerError::NoSuchDirectory)
    }

    fn piece_from_file_unit_test(
        file: &DirEntry,
        idx: u32,
        length: u32,
    ) -> Result<Option<Piece>, ServerError> {
        if let Ok(mut downloaded_file) = File::open(file.path()) {
            let pos = idx * length;
            if downloaded_file.seek(SeekFrom::Start(pos as u64)).is_ok() {
                let mut buffer = vec![0u8; length as usize];
                if downloaded_file.read_exact(&mut buffer).is_ok() {
                    let mut piece = Piece::new(idx, 0, vec![0u8; 20]);
                    piece.add_block(buffer);
                    return Ok(Some(piece));
                }
            }
        }
        Err(ServerError::NoSuchDirectory)
    }

    #[test]
    fn loading_correct_piece() {
        if let Ok(torrent) = TorrentInfo::new(
            "files_for_testing/torrents_testing/ubuntu-20.04.4-desktop-amd64.iso.torrent",
        ) {
            let exp_piece_hash = torrent.get_hash(0);

            if let Ok(Some(piece0)) = load_piece_for_unit_test(0) {
                let mut hasher = Sha1::new();
                hasher.update(piece0.get_data());
                let piece_hash = hasher.finalize();

                assert_eq!(exp_piece_hash, piece_hash.to_vec());
                return;
            }
        }
        assert!(false);
    }
}
