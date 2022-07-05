use std::fs::DirEntry;
use std::io::Seek;
use std::net::TcpStream;

use crate::bitfield::PieceBitfield;
use crate::bittorrent_client::piece::Piece;
use crate::bittorrent_client::torrent_info::TorrentInfo;
use crate::constants::*;
use crate::errors::ServerError;
use crate::p2p_messages::bitfield::BitfieldMessage;
use crate::p2p_messages::handshake::Handshake;
use crate::p2p_messages::message_builder::MessageBuilder;
use crate::p2p_messages::message_builder::P2PMessage;
use crate::p2p_messages::message_trait::Message;
use crate::p2p_messages::piece::PieceMessage;
use crate::p2p_messages::request::RequestMessage;
use crate::p2p_messages::unchoke::UnchokeMessage;

use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::SeekFrom;
use std::sync::{Arc, RwLock};
use std::time::Duration;

pub struct PeerConnection {
    stream: TcpStream,
    is_choked: u8,
    is_interested: u8,
    our_pieces: PieceBitfield,
    torrent_info: TorrentInfo,
    download_path: String,
    piece: Option<Piece>,
}

/// # struct PeerConnection (server)
/// Contains all information about the connection.
/// Fields:
//      - stream
//      - is_choked -> peer is choked
//      - is_interested -> peer is interested
//      - our_pieces -> PieceBitfield of our pieces
//      - torrent_info -> torrent that peer is interested in
//      - download_path
//      - piece -> piece requested by the peer.
impl PeerConnection {
    pub fn new(
        mut stream: TcpStream,
        torrents: Arc<RwLock<Vec<(TorrentInfo, PieceBitfield)>>>,
        download_path: String,
    ) -> Result<PeerConnection, ServerError> {
        if let Ok(handshake) = Handshake::read_msg(&mut stream) {
            if stream
                .set_read_timeout(Some(Duration::new(TWO_MINUTES, 0)))
                .is_ok()
            {
                let info_hash = handshake.get_info_hash();
                let (torrent_info, our_pieces) = get_torrent_info(&info_hash, torrents)?;
                PeerConnection::send_handshake(info_hash, &mut stream)?;

                return Ok(PeerConnection {
                    stream,
                    is_choked: CHOKED,
                    is_interested: NOT_INTERESTED,
                    our_pieces,
                    torrent_info,
                    download_path,
                    piece: None,
                });
            }
        }
        Err(ServerError::HandshakeError)
    }

    fn send_handshake(info_hash: Vec<u8>, stream: &mut TcpStream) -> Result<(), ServerError> {
        let our_handshake = Handshake::new_from_param(
            "BitTorrent protocol",
            info_hash,
            PEER_ID.as_bytes().to_vec(),
        );
        match our_handshake.send_msg(stream) {
            Ok(_) => Ok(()),
            Err(_) => Err(ServerError::HandshakeError),
        }
    }

    /// Sends the Bitfield message as the first message, then it listening for new messages
    /// When a new message from the other peer arrives, it is handled.
    pub fn handle_connection(&mut self) {
        if let Ok(bf_msg) = BitfieldMessage::new(self.our_pieces.get_vec()) {
            let _ = bf_msg.send_msg(&mut self.stream);
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
        if UnchokeMessage::new().send_msg(&mut self.stream).is_ok() {
            self.is_choked = UNCHOKED;
        }
    }

    /// Receives a Request Message, loads the piece in self.piece and then
    /// gets the correct block of bytes from this piece.
    fn handle_request(&mut self, msg: RequestMessage) {
        let piece_idx = msg.get_piece_index();
        if self.is_interested == NOT_INTERESTED
            || self.is_choked == CHOKED
            || !self.our_pieces.has_piece(piece_idx)
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
        if let Ok(msg) = PieceMessage::new(piece_idx, msg.get_begin(), block) {
            if msg.send_msg(&mut self.stream).is_ok() {}
        }
    }

    fn load_piece(&self, piece_idx: u32) -> Result<Option<Piece>, ServerError> {
        if let Ok(files) = fs::read_dir(&self.download_path) {
            for file in files {
                let file = file.unwrap();
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
            piece_file.read_to_end(&mut buffer).unwrap();
            piece.add_block(buffer);
            return Ok(Some(piece));
        }
        Err(ServerError::NoSuchDirectory)
    }

    fn piece_from_file(&self, file: &DirEntry, idx: u32) -> Result<Option<Piece>, ServerError> {
        if let Ok(mut downloaded_file) = File::open(file.path()) {
            let pos = idx * self.torrent_info.get_piece_length();
            if downloaded_file.seek(SeekFrom::Start(pos as u64)).is_ok() {
                let mut buffer = vec![0u8; self.torrent_info.length_of_piece_n(idx) as usize];
                downloaded_file.read_exact(&mut buffer).unwrap();
                let mut piece = Piece::new(idx, 0, vec![0u8; 20]);
                piece.add_block(buffer);
                return Ok(Some(piece));
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
}

fn get_torrent_info(
    info_hash: &[u8],
    torrents: Arc<RwLock<Vec<(TorrentInfo, PieceBitfield)>>>,
) -> Result<(TorrentInfo, PieceBitfield), ServerError> {
    if let Ok(vec_torrents) = torrents.read() {
        for torrent in &*vec_torrents {
            if torrent.0.get_info_hash() == *info_hash {
                return Ok((torrent.0.clone(), torrent.1.clone()));
            }
        }
    }
    Err(ServerError::CannotFindTorrent)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bittorrent_client::torrent_info::TorrentInfo;
    use sha1::{Digest, Sha1};

    fn load_piece_for_unit_test(piece_idx: u32) -> Result<Option<Piece>, ServerError> {
        let torrent_name = "ubuntu-20.04.4-desktop-amd64.iso";
        let download_dir = "files_for_testing/downloaded_files";
        let length = 262144;

        if let Ok(files) = fs::read_dir(download_dir) {
            for file in files {
                let file = file.unwrap();
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
            piece_file.read_to_end(&mut buffer).unwrap();
            piece.add_block(buffer);
            return Ok(Some(piece));
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
                downloaded_file.read_exact(&mut buffer).unwrap();
                let mut piece = Piece::new(idx, 0, vec![0u8; 20]);
                piece.add_block(buffer);
                return Ok(Some(piece));
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
