// use std::io::Seek;
// use std::net::TcpStream;

// use crate::bitfield::PieceBitfield;
// use crate::bittorrent_client::peer::Peer;
// use crate::bittorrent_client::piece_queue::Piece;
// use crate::bittorrent_client::torrent_info::TorrentInformation;
// use crate::constants::*;
// use crate::errors::ServerError;
// use crate::event_messages::NewEvent;
// use crate::p2p_messages::handshake::Handshake;
// use crate::p2p_messages::have::HaveMessage;
// use crate::p2p_messages::message_builder::MessageBuilder;
// use crate::p2p_messages::message_builder::P2PMessage;
// use crate::p2p_messages::message_trait::Message;
// use crate::p2p_messages::piece::PieceMessage;
// use crate::p2p_messages::request::RequestMessage;
// use crate::p2p_messages::unchoke::UnchokeMessage;
// use std::fs;
// use std::fs::File;
// use std::io::Read;
// use std::io::SeekFrom;
// use std::sync::mpsc::Receiver;
// use std::sync::{Arc, Mutex};
// use std::time::Duration;
// use std::collections::HashMap;

// pub struct PeerConnection {
//     stream: TcpStream,
//     rx: Arc<Mutex<Receiver<NewEvent>>>,
//     is_choked: u8,
//     is_interested: u8,
//     our_pieces: PieceBitfield,
//     torrent_info: TorrentInformation,
//     peer_id: Vec<u8>,
//     download_path: String,
//     piece: Option<Piece>,
// }

// impl PeerConnection {
//     pub fn new(
//         mut stream: TcpStream,
//         rx: Arc<Mutex<Receiver<NewEvent>>>,
//         torrents: &HashMap<Vec<u8>,TorrentInformation>,
//         download_path: String,
//     ) -> Result<PeerConnection, ServerError> {
//         if let Ok(handshake) = Handshake::read_msg(&mut stream) {
//             if stream.set_read_timeout(Some(Duration::new(TWO_MINUTES, 0))).is_ok() {
//                 let info_hash = handshake.get_info_hash();
//                 let torrent_info = get_torrent_info(&info_hash, torrents)?;
//                 PeerConnection::send_handshake(info_hash, &mut stream)?;

//                 return Ok(PeerConnection {
//                     stream,
//                     rx,
//                     is_choked: CHOKED,
//                     is_interested: NOT_INTERESTED,
//                     peer_id: handshake.get_peer_id(),
//                     our_pieces: torrent_info.get_bitfield(),
//                     torrent_info,
//                     download_path,
//                     piece: None,
//                 });
//             }
//         }
//         Err(ServerError::HandshakeError)
//     }

//     fn send_handshake(info_hash: Vec<u8>, stream: &mut TcpStream) -> Result<(), ServerError> {
//         let our_handshake = Handshake::new_from_param(
//             "BitTorrent protocol",
//             info_hash,
//             PEER_ID.as_bytes().to_vec(),
//         );
//         match our_handshake.send_msg(stream) {
//             Ok(_) => Ok(()),
//             Err(_) => Err(ServerError::HandshakeError),
//         }
//     }

//     pub fn handle_connection(&mut self) {
//         let (mut new_piece, mut idx, mut torrent_hash) = (false, 0, vec![]);

//         loop {
//             match MessageBuilder::build(&mut self.stream) {
//                 Ok(msg) => self.handle_msg(msg),
//                 Err(_) => break,
//             }

//             if let Ok(rx_sh) = self.rx.lock() {
//                 if let Ok(new_event) = rx_sh.try_recv() {
//                     match new_event {
//                         NewEvent::NewPiece(idx_, torrent_hash_) => {
//                             new_piece = true;
//                             idx = idx_;
//                             torrent_hash = torrent_hash_;
//                         }
//                         _ => (),
//                     }
//                 }
//             }

//             if new_piece{
//                 self.handle_new_piece(idx, torrent_hash.clone());
//             }
//         }
//     }

//     fn handle_new_piece(&mut self, idx: u32, torrent_hash:Vec<u8>) {
//         if self.torrent_info.get_info_hash() == torrent_hash {
//             HaveMessage::new(idx).send_msg(&mut self.stream);
//         }
//     }

//     fn handle_msg(&mut self, message: P2PMessage) {
//         match message {
//             P2PMessage::Interested(_msg) => self.handle_interested_msg(),
//             P2PMessage::NotInterested(_msg) => self.is_interested = NOT_INTERESTED,
//             P2PMessage::Request(msg) => self.handle_request(msg),
//             P2PMessage::Cancel(msg) => (),
//             _ => (),
//         }
//     }

//     fn handle_interested_msg(&mut self) {
//         self.is_interested = INTERESTED;
//         if UnchokeMessage::new().send_msg(&mut self.stream).is_ok() {
//             self.is_choked = UNCHOKED;
//         }
//     }

//     fn handle_request(&mut self, msg: RequestMessage) {
//         let piece_idx = msg.get_piece_index();
//         if self.is_interested == NOT_INTERESTED
//             || self.is_choked == CHOKED
//             || !self.our_pieces.has_piece(piece_idx)
//         {
//             return;
//         }

//         if let Some(piece) = &self.piece {
//             if piece.get_idx() != piece_idx {
//                 match self.load_piece(piece_idx) {
//                     Ok(piece) => self.piece = piece,
//                     Err(_) => return,
//                 }
//             }
//         } else if let None = self.piece {
//             match self.load_piece(piece_idx) {
//                 Ok(piece) => self.piece = piece,
//                 Err(_) => return,
//             }
//         }

//         let block = self.get_block(msg.get_begin(), msg.get_block_length());
//         if let Ok(msg) = PieceMessage::new(piece_idx, msg.get_begin(), block) {
//             msg.send_msg(&mut self.stream);
//         }
//         // Ok(())
//     }

//     fn load_piece(&self, piece_idx: u32) -> Result<Option<Piece>, ServerError> {
//         if let Ok(mut files) = fs::read_dir(&self.download_path) {
//             for file in files {
//                 let file = file.unwrap();
//                 let file_name = file.file_name().to_string_lossy().to_string();
//                 let piece_name = format!("{}_piece_{}", self.torrent_info.get_name(), piece_idx);

//                 // Get piece from directory
//                 if file_name == piece_name {
//                     if let Ok(mut piece_file) = File::open(file.path()) {
//                         let mut piece = Piece::new(piece_idx, 0, vec![0u8; 20]);
//                         let mut buffer = Vec::new();
//                         piece_file.read_to_end(&mut buffer).unwrap();
//                         piece.add_block(buffer);
//                         return Ok(Some(piece));
//                     }
//                 }
//                 // Get piece from a entire downloaded file
//                 else if file_name == self.torrent_info.get_name() {
//                     if let Ok(mut downloaded_file) = File::open(file.path()) {
//                         let pos = piece_idx * self.torrent_info.get_piece_length();
//                         downloaded_file.seek(SeekFrom::Start(pos as u64));
//                         let mut buffer =
//                             vec![0u8; self.torrent_info.length_of_piece_n(piece_idx) as usize];
//                         downloaded_file.read_exact(&mut buffer).unwrap();
//                         let mut piece = Piece::new(piece_idx, 0, vec![0u8; 20]);
//                         piece.add_block(buffer);
//                         return Ok(Some(piece));
//                     }
//                 }
//             }
//         }
//         Err(ServerError::NoSuchDirectory)
//     }

//     fn get_block(&self, begin: u32, block_length: u32) -> Vec<u8> {
//         let mut block = Vec::new();
//         if let Some(piece) = &self.piece{
//             let piece_data = piece.get_data();
//             for i in begin..begin + block_length {
//                 block.push(piece_data[i as usize]);
//             }
//         }
//         block
//     }
// }

// fn get_torrent_info(
//     info_hash: &Vec<u8>,
//     torrents: &HashMap<Vec<u8>,TorrentInformation>,
// ) -> Result<TorrentInformation, ServerError> {
//     match torrents.get(info_hash){
//         Some(torrent) => Ok(torrent.clone()),
//         None => Err(ServerError::CannotFindTorrent),
//     }

// }
