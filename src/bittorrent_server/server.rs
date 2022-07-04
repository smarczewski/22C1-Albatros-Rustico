// use crate::bittorrent_client::torrent_info::TorrentInformation;
// use crate::event_messages::NewEventMessage;
// use crate::p2p_messages::choke::ChokeMessage;
// use crate::p2p_messages::keep_alive::KeepAliveMessage;
// use crate::p2p_messages::message_builder::{MessageBuilder, P2PMessage};
// use crate::p2p_messages::message_trait::Message;
// use crate::p2p_messages::unchoke::UnchokeMessage;
// use crate::thread_mgmt::threadpool::ThreadPool;
// use crate::bitfield::PieceBitfield;

// use std::collections::HashMap;
// use std::io::{Error, ErrorKind};
// use std::net::{TcpListener, TcpStream};
// use std::sync::mpsc::{Receiver,Sender};
// use std::thread;
// use std::sync::{mpsc, Arc, Mutex};

// use super::peer_connection::PeerConnection;

// pub struct Server {
//     tcp_port: String,
//     pub pool: ThreadPool,
//     torrents: HashMap<Vec<u8>, TorrentInformation>,
//     rx_client: Receiver<NewEventMessage>,
// }

// impl Server {
//     pub fn new(settings: &HashMap<String, String>, rx_client: Receiver<NewEventMessage>, torrents: HashMap<Vec<u8>, TorrentInformation>) -> Result<Server, Error> {
//         let tcp_port = settings.get(&"tcp_port".to_string());
//         let pool = ThreadPool::new(4);
//         match tcp_port {
//             Some(p) => Ok(Server {
//                 tcp_port: p.clone(),
//                 pool,
//                 torrents,
//                 rx_client,
//             }),
//             _ => Err(Error::new(ErrorKind::InvalidInput, "Invalid settings")),
//         }
//     }

//     pub fn run_server(self) -> Result<(), Error> {
//         let listener = TcpListener::bind("127.0.0.1:".to_string() + &self.tcp_port)?;
//         let (tx, rx) = mpsc::channel();
//         let sh_rx = Arc::new(Mutex::new(rx));

//         for stream in listener.incoming() {
//             let shrx1 = sh_rx.clone();
//             let stream = stream.unwrap();

//             // self.pool.execute(|| {
//             //     handle_connection(stream);
//             // });
//             let path = "downloaded_files".to_string();
//             thread::spawn(|| {
//                 if let Ok(peer_connection) = PeerConnection::new(stream, sh_rx,&self.torrents, path){
//                     peer_connection.handle_connection();
//                 }
//             });
//         }

//         loop{
//             if let Ok(new_event) = self.rx_client.recv(){
//                 if let NewEventMessage::NewPiece(idx, torrent_hash) = new_event{
//                     self.handle_new_piece(idx, torrent_hash, Sender::clone(&tx));
//                 }
//             }
//         }
//     }

//     fn handle_new_piece(&mut self, idx: u32, torrent_hash: Vec<u8>, tx_peer: Sender<NewEventMessage>) {
//         if let Some(torrent) = self.torrents.get(&torrent_hash){
//             let bitfield = torrent.get_bitfield();
//             bitfield.add_a_piece(idx);
//         }
//         tx_peer.send(NewEventMessage::NewPiece(idx, torrent_hash));
//     }
// }

// #[cfg(test)]
// mod tests {
//     // use super::*;
//     // use crate::encoding_decoding::settings_parser::SettingsParser;

//     // #[test]
//     // fn server_is_created_correctly() {
//     //     let settings = SettingsParser
//     //         .parse_file("files_for_testing/settings_files_testing/valid_format_v2.txt")
//     //         .unwrap();
//     //     let server = Server::new(&settings);
//     //     assert!(server.is_ok());
//     // }

//     // #[test]
//     // fn server_doesnt_run_on_invalid_port() {
//     //     let settings = SettingsParser
//     //         .parse_file("files_for_testing/settings_files_testing/valid_format_invalid_port.txt")
//     //         .unwrap();
//     //     let server = Server::new(&settings).unwrap();
//     //     assert!(server.run_server().is_err());
//     // }
// }
