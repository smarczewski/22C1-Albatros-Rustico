// // ThreadPool as implemented on The Rust Programming Language Book Chapter 20

// use crate::bittorrent_client::client::Client;
// use crate::bittorrent_client::peer::Peer;
// use crate::bittorrent_client::peer_connection::PeerConnection;
// use crate::bittorrent_client::piece_queue::Piece;
// use crate::errors::*;
// use crate::event_messages::NewEvent;
// use std::fs::{self, File};
// use std::io::Write;
// use std::path::Path;
// use std::process::id;
// use std::sync::mpsc::{self, Receiver, Sender};
// use std::sync::{Arc, Mutex, RwLock};
// use std::thread;

// // A thread pool is a group of spawned threads that are waiting and ready to handle a task.
// pub struct ClientThreadPool {
//     workers: Vec<ClientWorker>,
//     sender: mpsc::Sender<TaskMessage>,
// }

// enum TaskMessage {
//     DownloadPiece(Piece),
//     Terminate,
// }

// impl ClientThreadPool {
//     pub fn new(
//         size: usize,
//         peers: Arc<RwLock<Vec<Peer>>>,
//         client: Arc<Client>,
//         tx_client: Sender<NewEvent>,
//     ) -> ClientThreadPool {
//         let (sender, receiver) = mpsc::channel();
//         let receiver = Arc::new(Mutex::new(receiver));
//         let mut workers = Vec::with_capacity(size);

//         for id in 0..size {
//             workers.push(ClientWorker::new(
//                 id,
//                 Arc::clone(&receiver),
//                 peers.clone(),
//                 client.clone(),
//                 Sender::clone(&tx_client),
//             ));
//         }

//         ClientThreadPool { workers, sender }
//     }

//     pub fn download(&self, piece: Piece) {
//         if self.sender.send(TaskMessage::DownloadPiece(piece)).is_err() {
//             println!("Thread pool cannot send a piece to workers");
//         }
//     }
// }

// impl Drop for ClientThreadPool {
//     fn drop(&mut self) {
//         for _ in &self.workers {
//             self.sender
//                 .send(TaskMessage::Terminate)
//                 .expect("Cannot send Terminate announce to workers");
//         }

//         for worker in &mut self.workers {
//             println!("Shutting down worker: {}", worker.id);
//             if let Some(thread) = worker.thread.take() {
//                 thread.join().expect("Error joining the worker threads");
//             }
//         }
//     }
// }

// struct ClientWorker {
//     id: usize,
//     thread: Option<thread::JoinHandle<()>>,
// }

// // impl ClientWorker {
// //     fn new(
// //         id: usize,
// //         receiver: Arc<Mutex<Receiver<TaskMessage>>>,
// //         peers: Arc<RwLock<Vec<Peer>>>,
// //         client: Arc<Client>,
// //         tx_cl: Sender<NewEvent>,
// //     ) -> ClientWorker {
// //         let mut curr_connection:Option<PeerConnection> = None;
// //         let tx_sh = tx_cl.clone();

// //         let thread = thread::spawn(move || loop {

// //             let message = receiver.lock().unwrap().recv().unwrap();
// //             match message {
// //                 TaskMessage::DownloadPiece(mut piece) => {
// //                     while curr_connection.is_none(){
// //                         let peer = peers.write().unwrap().pop();
// //                         println!("Saco peer {}", id);
// //                         if let Some(new_peer) = peer{
// //                             if let Ok(mut new_connection) = PeerConnection::new(new_peer, client.clone()){
// //                                 if new_connection.first_contact().is_ok(){
// //                                     curr_connection = Some(new_connection);
// //                                     println!("Conexion nueva! {}",id);
// //                                 }
// //                             }
// //                         }else{
// //                             thread::yield_now();
// //                         }
// //                     }

// //                     if let Some(peer_connection) = &mut curr_connection{
// //                         match peer_connection.download_piece(&mut piece, id){
// //                             Ok(_) => {
// //                                 println!("Nueva pieza: {} - id {}", piece.get_idx(), id);
// //                                 tx_cl.send(NewEvent::NewDownloadedPiece(piece)).unwrap();},
// //                             Err(DownloadError::CannotReadPeerMessage) => tx_cl.send(NewEvent::CannotDownloadPiece(piece)).unwrap(),
// //                             _ => (),
// //                         }
// //                     }
// //                 },
// //                 TaskMessage::Terminate => {
// //                     //droppear connecion
// //                     break;
// //                 }
// //             }
// //         });
// //         ClientWorker {
// //             id,
// //             thread: Some(thread),
// //         }
// //     }
// // }

// // /// If there is a connection, it does nothing.
// // /// Else, it tries to connect to some peer from the list.
// // ///     If it succeeds, the value of current connection is updated.
// // ///     Otherwise, if there are no more peers, peer connection value will remains None.
// // fn check_connection(
// //     curr_connection: &mut Option<PeerConnection>,
// //     peers: &Arc<RwLock<Vec<Peer>>>,
// //     client: Arc<Client>,
// //     tx: Sender<NewEvent>,
// //     id: usize,
// // ) {
// //     loop {
// //         if curr_connection.is_some() {
// //             break;
// //         }
// //         println!("buscando peer {}", id);
// //         match establish_connection(&peers, client.clone(), id) {
// //             Ok(peer_connection) => {
// //                 *curr_connection = Some(peer_connection);
// //                 tx.send(NewEvent::NewConnection);
// //             }
// //             Err(DownloadError::NoPeers) => {
// //                 println!("no peers");
// //                 thread::yield_now();
// //                 break;
// //             }
// //             _ => (),
// //         }
// //     }
// // }

// // /// Starts the download of a piece and handles the result.
// // /// If the download succeeds, it just announces to the client about the new downloades piece.
// // /// On error, it makes some decission according to the error type:
// // ///     - If the peer stopped sending messages -> returns the piece to client and closes the connection
// // ///     - Another error, for example, peer has not the piece -> returns the piece to client, but we
// // ///     will remain connected to it.
// // fn handle_download(
// //     connection: &mut Option<PeerConnection>,
// //     client: Arc<Client>,
// //     tx: Sender<NewEvent>,
// //     mut piece: Piece,
// //     id: usize,
// // ) {
// //     if let Some(peer_conn) = connection {
// //         match peer_conn.download_piece(&mut piece, id) {
// //             Ok(_) => match write_piece_in_file(&mut piece, client) {
// //                 Ok(_) => announce_downloaded_piece(piece, Sender::clone(&tx)),
// //                 Err(_) => return_piece(piece, Sender::clone(&tx)),
// //             },

// //             Err(DownloadError::CannotReadPeerMessage) => {
// //                 println!("Error protocolo - {}", id);
// //                 *connection = None;
// //                 println!("Piece:\n{:?}\n", piece);
// //                 return_piece(piece, Sender::clone(&tx));
// //                 if tx.send(NewEvent::ConnectionDropped).is_err() {
// //                     println!("Error at announcing the connection drop");
// //                 }
// //             }

// //             _ => return_piece(piece, Sender::clone(&tx)),
// //         }
// //     }
// // }

// // fn establish_connection(
// //     peers: &Arc<RwLock<Vec<Peer>>>,
// //     client: Arc<Client>,
// //     id: usize,
// // ) -> Result<PeerConnection, DownloadError> {
// //     if let Some(curr_peer) = get_a_peer(peers, id) {
// //         if let Ok(mut peer_conn) = PeerConnection::new(curr_peer, client.clone()) {
// //             if peer_conn.first_contact().is_ok() {
// //                 return Ok(peer_conn);
// //             }
// //         }
// //         return Err(DownloadError::ConnectionFailed);
// //     }
// //     Err(DownloadError::NoPeers)
// // }

// // fn get_a_peer(peers: &Arc<RwLock<Vec<Peer>>>, id: usize) -> Option<Peer> {
// //     let mut sh_peers = peers.write().unwrap();
// //     return sh_peers.pop();
// //     // if let Ok(mut sh_peers) = peers.write() {
// //     // println!("Get a peer - {}", id);
// //     // return sh_peers.pop();
// //     // }
// //     // None
// // }

// // fn write_piece_in_file(piece: &mut Piece, client: Arc<Client>) -> Result<(), ()> {
// //     println!("DOWNLOADED PIECE: {}", piece.get_idx());
// //     let download_dir_path = client.get_download_dir();
// //     let torrent_name = client.get_torrent_info().get_name();

// //     if !Path::new(&download_dir_path).exists() {
// //         if fs::create_dir_all(&download_dir_path).is_err() {
// //             return Err(());
// //         }
// //     }

// //     let path = format!(
// //         "{}/{}_piece_{}",
// //         download_dir_path,
// //         torrent_name,
// //         piece.get_idx(),
// //     );
// //     if let Ok(mut file) = File::create(path) {
// //         if file.write_all(&piece.get_data()).is_ok() {
// //             return Ok(());
// //         }
// //     }
// //     Err(())
// // }

// // fn return_piece(piece: Piece, tx: Sender<NewEvent>) {
// //     if tx.send(NewEvent::CannotDownloadPiece(piece)).is_err() {
// //         println!("The piece was lost");
// //     }
// // }

// // fn announce_downloaded_piece(piece: Piece, tx: Sender<NewEvent>) {
// //     if tx.send(NewEvent::NewDownloadedPiece(piece)).is_err() {
// //         println!("Error announcing the download of a new piece");
// //     }
// // }

// impl ClientWorker {
//     fn new(
//         id: usize,
//         receiver: Arc<Mutex<Receiver<TaskMessage>>>,
//         peers: Arc<RwLock<Vec<Peer>>>,
//         client: Arc<Client>,
//         tx_cl: Sender<NewEvent>,
//     ) -> ClientWorker {
//         let mut curr_connection = None;
//         let tx_sh = tx_cl.clone();

//         let thread = thread::spawn(move || loop {
//             // println!("Esperando {}", id);
//             let message = receiver.lock().unwrap().recv().unwrap();
//             // println!("Lei {}", id);
//             match message {
//                 TaskMessage::DownloadPiece(piece) => {
//                     check_connection(
//                         &mut curr_connection,
//                         &peers,
//                         client.clone(),
//                         tx_sh.clone(),
//                         id,
//                     );
//                     handle_download(
//                         &mut curr_connection,
//                         client.clone(),
//                         tx_sh.clone(),
//                         piece,
//                         id,
//                     );
//                 }
//                 TaskMessage::Terminate => {
//                     //droppear connecion
//                     break;
//                 }
//             }
//         });
//         ClientWorker {
//             id,
//             thread: Some(thread),
//         }
//     }
// }

// /// If there is a connection, it does nothing.
// /// Else, it tries to connect to some peer from the list.
// ///     If it succeeds, the value of current connection is updated.
// ///     Otherwise, if there are no more peers, peer connection value will remains None.
// fn check_connection(
//     curr_connection: &mut Option<PeerConnection>,
//     peers: &Arc<RwLock<Vec<Peer>>>,
//     client: Arc<Client>,
//     tx: Sender<NewEvent>,
//     id: usize,
// ) {
//     loop {
//         if curr_connection.is_some() {
//             break;
//         }
//         println!("buscando peer {}", id);
//         match establish_connection(&peers, client.clone(), id) {
//             Ok(peer_connection) => {
//                 *curr_connection = Some(peer_connection);
//                 tx.send(NewEvent::NewConnection);
//             }
//             Err(DownloadError::NoPeers) => {
//                 println!("no peers");
//                 thread::yield_now();
//                 break;
//             }
//             _ => (),
//         }
//     }
// }

// /// Starts the download of a piece and handles the result.
// /// If the download succeeds, it just announces to the client about the new downloades piece.
// /// On error, it makes some decission according to the error type:
// ///     - If the peer stopped sending messages -> returns the piece to client and closes the connection
// ///     - Another error, for example, peer has not the piece -> returns the piece to client, but we
// ///     will remain connected to it.
// fn handle_download(
//     connection: &mut Option<PeerConnection>,
//     client: Arc<Client>,
//     tx: Sender<NewEvent>,
//     mut piece: Piece,
//     id: usize,
// ) {
//     if let Some(peer_conn) = connection {
//         match peer_conn.download_piece(&mut piece, id) {
//             Ok(_) => match write_piece_in_file(&mut piece, client) {
//                 Ok(_) => announce_downloaded_piece(piece, Sender::clone(&tx)),
//                 Err(_) => return_piece(piece, Sender::clone(&tx)),
//             },

//             Err(DownloadError::CannotReadPeerMessage) => {
//                 println!("Error protocolo - {}", id);
//                 *connection = None;
//                 println!("Piece:\n{:?}\n", piece);
//                 return_piece(piece, Sender::clone(&tx));
//                 if tx.send(NewEvent::ConnectionDropped).is_err() {
//                     println!("Error at announcing the connection drop");
//                 }
//             }

//             _ => return_piece(piece, Sender::clone(&tx)),
//         }
//     }
// }

// fn establish_connection(
//     peers: &Arc<RwLock<Vec<Peer>>>,
//     client: Arc<Client>,
//     id: usize,
// ) -> Result<PeerConnection, DownloadError> {
//     if let Some(curr_peer) = get_a_peer(peers, id) {
//         if let Ok(mut peer_conn) = PeerConnection::new(curr_peer, client.clone()) {
//             if peer_conn.first_contact().is_ok() {
//                 return Ok(peer_conn);
//             }
//         }
//         return Err(DownloadError::ConnectionFailed);
//     }
//     Err(DownloadError::NoPeers)
// }

// fn get_a_peer(peers: &Arc<RwLock<Vec<Peer>>>, id: usize) -> Option<Peer> {
//     let mut sh_peers = peers.write().unwrap();
//     return sh_peers.pop();
//     // if let Ok(mut sh_peers) = peers.write() {
//     // println!("Get a peer - {}", id);
//     // return sh_peers.pop();
//     // }
//     // None
// }

// fn write_piece_in_file(piece: &mut Piece, client: Arc<Client>) -> Result<(), ()> {
//     println!("DOWNLOADED PIECE: {}", piece.get_idx());
//     let download_dir_path = client.get_download_dir();
//     let torrent_name = client.get_torrent_info().get_name();

//     if !Path::new(&download_dir_path).exists() {
//         if fs::create_dir_all(&download_dir_path).is_err() {
//             return Err(());
//         }
//     }

//     let path = format!(
//         "{}/{}_piece_{}",
//         download_dir_path,
//         torrent_name,
//         piece.get_idx(),
//     );
//     if let Ok(mut file) = File::create(path) {
//         if file.write_all(&piece.get_data()).is_ok() {
//             return Ok(());
//         }
//     }
//     Err(())
// }

// fn return_piece(piece: Piece, tx: Sender<NewEvent>) {
//     if tx.send(NewEvent::CannotDownloadPiece(piece)).is_err() {
//         println!("The piece was lost");
//     }
// }

// fn announce_downloaded_piece(piece: Piece, tx: Sender<NewEvent>) {
//     if tx.send(NewEvent::NewDownloadedPiece(piece)).is_err() {
//         println!("Error announcing the download of a new piece");
//     }
// }
