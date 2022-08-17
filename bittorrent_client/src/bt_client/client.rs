use super::piece_queue::PieceQueue;
use crate::bencode_type::BencodeType;
use crate::bitfield::PieceBitfield;
use crate::bt_client::peer::Peer;
use crate::bt_client::peer_connection::PeerConnection;
use crate::bt_client::tracker_request::TrackerRequest;
use crate::constants::*;
use crate::errors::*;
use crate::event_messages::NewEvent;
use crate::logging::msg_coder::MsgCoder;
use crate::piece::Piece;
use crate::piece_merger::PieceMerger;
use crate::settings::Settings;
use crate::torrent_info::TorrentInfo;

use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;
use std::thread;
use std::thread::JoinHandle;
use std::vec;

/// # struct PeerConnection
/// Represents the BitTorrent client.
/// Fields:
///     - settings
///     - peer id
///     - torrent -> parsed torrent file
///     - downloaded_pieces: bitfield with out pieces
///     - tx_logger
#[derive(Debug, Clone)]
pub struct Client {
    settings: Arc<Settings>,
    our_id: Vec<u8>,
    torrent: TorrentInfo,
    downloaded_pieces: Arc<RwLock<PieceBitfield>>,
    tx_logger: Sender<String>,
    tx_gui: glib::Sender<NewEvent>,
}

impl Client {
    /// Creates and runs a client.
    pub fn init(
        settings: Arc<Settings>,
        torrent: (TorrentInfo, Arc<RwLock<PieceBitfield>>),
        tx_logger: Sender<String>,
        rx_gui: Arc<Mutex<Receiver<glib::Sender<NewEvent>>>>,
    ) -> Result<(), (TorrentInfo, Arc<RwLock<PieceBitfield>>)> {
        let mut client = Client::new(settings, torrent.0, torrent.1, tx_logger, rx_gui);
        let _ = client.tx_gui.send(NewEvent::DownloadingTorrent(
            client.get_torrent_info().get_name(),
        ));

        match client.run_client() {
            Ok(_) => {
                println!(
                    "Torrent: {} has been downloaded successfully",
                    client.get_torrent_info().get_name()
                );
                Ok(())
            }
            Err(error) => {
                error.print_error();
                let _ = client.tx_gui.send(NewEvent::TorrentDownloadFailed(
                    client.get_torrent_info().get_name(),
                ));
                Err((client.get_torrent_info(), client.get_dl_pieces()))
            }
        }
    }

    /// Receives the settings and the path of the torrent file.
    /// On success, returns a client which is correctly initialized
    pub fn new(
        settings: Arc<Settings>,
        torrent: TorrentInfo,
        downloaded_pieces: Arc<RwLock<PieceBitfield>>,
        tx_logger: Sender<String>,
        rx_gui: Arc<Mutex<Receiver<glib::Sender<NewEvent>>>>,
    ) -> Client {
        let tx_gui = rx_gui.lock().unwrap().recv().unwrap();

        Client {
            settings,
            our_id: CLIENT_ID.as_bytes().to_vec(),
            torrent,
            downloaded_pieces,
            tx_logger,
            tx_gui,
        }
    }

    /// The client runs. It implies:
    ///     - Client connects to tracker, sends the request and gets the tracker response.
    ///     - Gets peer list
    ///     - Chooses one peer, then connects to it
    ///     - A PeerConnection is created
    ///     - Download starts.
    ///     - On success, all pieces are joined.
    ///
    /// On error, it returns ClientError::DownloadError
    pub fn run_client(&mut self) -> Result<(), ClientError> {
        if self.file_is_downloaded() {
            let _ = self.connect_to_tracker(self.torrent.get_n_pieces());
            return Ok(());
        }

        let piece_queue = PieceQueue::new(&self.torrent, &self.downloaded_pieces);
        let response =
            self.connect_to_tracker(self.torrent.get_n_pieces() - piece_queue.length())?;
        let peer_list = self.get_peer_list(&response)?;
        self.notify_no_of_peers(peer_list.len() as u32);

        let mut vec_threads: Vec<JoinHandle<()>> = vec![];
        let (tx, rx) = mpsc::channel();
        let dl_finished = Arc::new(RwLock::new(false));
        let sh_piece_queue = Arc::new(RwLock::new(piece_queue));
        for peer in peer_list {
            let thread = self.handle_connection(
                self.clone(),
                peer,
                Sender::clone(&tx),
                sh_piece_queue.clone(),
                dl_finished.clone(),
            );
            vec_threads.push(thread);
        }
        self.listen_for_new_events(rx, dl_finished);
        self.join_peer_conn_threads(vec_threads)?;

        if self.file_is_downloaded() && self.merge_pieces().is_ok() {
            let _ = self.connect_to_tracker(self.torrent.get_n_pieces());
            return Ok(());
        }

        Err(ClientError::DownloadError)
    }

    /// Checks if the file has been already downloaded
    fn file_is_downloaded(&self) -> bool {
        if let Ok(dl_pieces) = self.downloaded_pieces.read() {
            return dl_pieces.has_all_pieces();
        }
        false
    }

    /// The client connects to tracker, sends the request and receives the response.
    /// On success, returns the response.
    /// Otherwise, return an error.
    fn connect_to_tracker(&mut self, n_dl_pieces: u32) -> Result<BencodeType, ClientError> {
        if let Ok(dl_pieces) = self.downloaded_pieces.read() {
            let piece_length = self.torrent.get_piece_length();
            let mut downloaded_bytes: u64 = (n_dl_pieces * piece_length) as u64;
            let last_piece = self.torrent.get_n_pieces() - 1;
            if dl_pieces.has_piece(last_piece) {
                downloaded_bytes -= piece_length as u64;
                downloaded_bytes += self.torrent.length_of_piece_n(last_piece) as u64;
            }

            let request = TrackerRequest::new(self, downloaded_bytes);
            match request.make_request() {
                Ok(response) => {
                    println!(
                        "\nConnected to the tracker. The response has been obtained successfully :)"
                    );
                    self.log_tracker_connection();
                    return Ok(response);
                }
                Err(_error) => return Err(ClientError::TrackerConnectionError),
            }
        }
        Err(ClientError::TrackerConnectionError)
    }

    /// Gets the peer list from the tracker response.
    fn get_peer_list(&self, response: &BencodeType) -> Result<Vec<Peer>, ClientError> {
        if let Ok(peers_benc) = response.get_value_from_dict("peers") {
            let mut peer_list = vec![];
            if let Ok(peer_list_aux) = peers_benc.get_list() {
                peer_list = peer_list_aux;
            } else if let Ok(peer_list_aux) = peers_benc.get_string() {
                if let Ok(peer_compacted_list) = self.parse_compacted_peers(peer_list_aux) {
                    peer_list = peer_compacted_list;
                }
            }
            if !peer_list.is_empty() {
                let mut peers = vec![];
                for peer in peer_list {
                    if let Ok(new_peer) = Peer::new(peer) {
                        peers.push(new_peer);
                    }
                }
                return Ok(peers);
            }
        }
        println!("The tracker response is invalid. Cannot continue :(");
        Err(ClientError::InvalidTrackerResponse)
    }

    /// Parses a compacted peer list
    fn parse_compacted_peers(
        &self,
        peers_compact: Vec<u8>,
    ) -> Result<Vec<BencodeType>, ClientError> {
        let mut list = vec![];
        let mut idx = 0;
        loop {
            if idx + 5 >= peers_compact.len() {
                return Ok(list);
            }

            let ip_result: Result<[u8; 4], _> = peers_compact[idx..idx + 4].try_into();
            let port_result: Result<[u8; 2], _> = peers_compact[idx + 4..idx + 6].try_into();

            if let (Ok(ip_aux), Ok(port_aux)) = (ip_result, port_result) {
                let port = u16::from_be_bytes(port_aux);
                let ip = format!("{}.{}.{}.{}", ip_aux[0], ip_aux[1], ip_aux[2], ip_aux[3]);
                idx += 6;

                let mut peer = HashMap::new();
                let peer_ip = BencodeType::String(ip.into_bytes());
                let peer_port = BencodeType::Integer(port as i64);

                peer.insert("ip".to_string(), peer_ip);
                peer.insert("port".to_string(), peer_port);

                list.push(BencodeType::Dictionary(peer));
            }
        }
    }

    /// Receives a peer and spawns a thread for each of these. Then, it tries to connect to the peer.
    /// On success, it starts the download.
    fn handle_connection(
        &self,
        client: Client,
        peer: Peer,
        tx: Sender<NewEvent>,
        piece_queue: Arc<RwLock<PieceQueue>>,
        dl_finished: Arc<RwLock<bool>>,
    ) -> JoinHandle<()> {
        let dl_pieces = self.downloaded_pieces.clone();

        thread::spawn(move || {
            let peer_connection =
                PeerConnection::new(client, peer, piece_queue.clone(), Sender::clone(&tx));

            if let Ok(mut new_peer_connection) = peer_connection {
                new_peer_connection.start_download(dl_pieces, dl_finished);
            }
        })
    }

    /// Client listens for new events:
    ///     - New piece was downloaded
    ///     - New connection
    ///     - A connection was dropped
    /// The client makes a decision according to the received event.
    /// The client stops listening for new events when it receives
    /// all pieces of the file or the connections counterreachs zero.
    fn listen_for_new_events(&mut self, rx: Receiver<NewEvent>, dl_finished: Arc<RwLock<bool>>) {
        let mut connection_counter = 0;
        let mut dl_pieces_counter = 0;

        loop {
            println!("Active connections: {}", connection_counter);
            if let Ok(new_event_msg) = rx.recv() {
                match new_event_msg {
                    NewEvent::NewConnection(torrent_name, peer) => {
                        self.handle_new_conn_msg(&mut connection_counter, torrent_name, peer);
                    }
                    NewEvent::NewDownloadedPiece(torrent_name, piece, peer) => {
                        dl_pieces_counter += 1;
                        self.handle_new_dl_piece_msg(torrent_name, piece, peer);
                    }
                    NewEvent::ConnectionDropped(torrent_name, peer) => {
                        self.handle_conn_dropped_msg(&mut connection_counter, torrent_name, peer);
                    }
                    NewEvent::OurStatus(status, peer) => {
                        self.handle_status_msg(status, peer);
                    }
                    _ => (),
                }
            }

            if dl_pieces_counter == self.get_torrent_info().get_n_pieces() {
                if let Ok(mut lock_dl) = dl_finished.write() {
                    *lock_dl = true;
                }
            }

            if connection_counter == 0 {
                break;
            }
        }
    }

    /// When receives a new connection message, it increases the active connections counter
    /// and notify GUI about this event.
    /// Also, this event is logged.
    fn handle_new_conn_msg(&self, conn_counter: &mut u32, torrent_name: String, peer: Peer) {
        *conn_counter += 1;
        self.log_peer_connection(&peer);
        if self
            .tx_gui
            .send(NewEvent::NewConnection(torrent_name, peer))
            .is_err()
        {
            let _ = self.tx_logger.send(MsgCoder::generate_message(
                ERROR_LOG_TYPE,
                CLIENT_MODE_LOG,
                "Failed to notify GUI about a new peer connection".to_string(),
            ));
        }
    }

    /// When receives a new downloaded piece message, it updates the bitfield
    /// and notify GUI about this event.
    /// Also, this event is logged.
    fn handle_new_dl_piece_msg(&self, torrent_name: String, piece: Piece, peer: Peer) {
        if let Ok(mut lock_dl) = self.downloaded_pieces.write() {
            lock_dl.add_a_piece(piece.get_idx());
        }
        self.log_downloaded_piece(piece.get_idx());

        if self
            .tx_gui
            .send(NewEvent::NewDownloadedPiece(torrent_name, piece, peer))
            .is_err()
        {
            let _ = self.tx_logger.send(MsgCoder::generate_message(
                ERROR_LOG_TYPE,
                CLIENT_MODE_LOG,
                "Failed to notify GUI about a new downloaded piece".to_string(),
            ));
        }
    }

    /// When receives a new connection message, it decreases the active connections counter
    /// and notify GUI about this event.
    fn handle_conn_dropped_msg(&self, conn_counter: &mut u32, torrent_name: String, peer: Peer) {
        *conn_counter -= 1;
        if self
            .tx_gui
            .send(NewEvent::ConnectionDropped(torrent_name, peer))
            .is_err()
        {
            let _ = self.tx_logger.send(MsgCoder::generate_message(
                ERROR_LOG_TYPE,
                CLIENT_MODE_LOG,
                "Failed to notify GUI about peer connection drop".to_string(),
            ));
        }
    }

    fn handle_status_msg(&self, status: String, peer: Peer) {
        if self.tx_gui.send(NewEvent::OurStatus(status, peer)).is_err() {
            let _ = self.tx_logger.send(MsgCoder::generate_message(
                ERROR_LOG_TYPE,
                CLIENT_MODE_LOG,
                "Failed to notify GUI about new status".to_string(),
            ));
        }
    }

    fn notify_no_of_peers(&self, no_of_peers: u32) {
        let torrent_name = self.get_torrent_info().get_name();
        if self
            .tx_gui
            .send(NewEvent::NumberOfPeers(torrent_name, no_of_peers))
            .is_err()
        {
            let _ = self.tx_logger.send(MsgCoder::generate_message(
                ERROR_LOG_TYPE,
                CLIENT_MODE_LOG,
                "Failed to notify GUI about number of peers provided by the tracker".to_string(),
            ));
        }
    }

    /// Logs tracker connection
    fn log_tracker_connection(&self) {
        if self
            .tx_logger
            .send(MsgCoder::generate_message(
                START_LOG_TYPE,
                CLIENT_MODE_LOG,
                format!(
                    "Connected to tracker: {} successfully\n",
                    self.torrent.get_announce()
                ),
            ))
            .is_err()
        {
            println!("Failed to log tracker connection");
        }
    }

    /// Logs peer connection.
    fn log_peer_connection(&self, peer: &Peer) {
        if self
            .tx_logger
            .send(MsgCoder::generate_message(
                GENERIC_LOG_TYPE,
                CLIENT_MODE_LOG,
                format!(
                    "Torrent: {} - Connected to new peer - {}:{}\n",
                    self.torrent.get_name(),
                    peer.ip(),
                    peer.port()
                ),
            ))
            .is_err()
        {
            println!("Failed to log peer connection");
        }
    }

    /// Logs piece downloading.
    fn log_downloaded_piece(&self, idx: u32) {
        if self
            .tx_logger
            .send(MsgCoder::generate_message(
                GENERIC_LOG_TYPE,
                CLIENT_MODE_LOG,
                format!(
                    "Torrent: {} - Piece #{} has been downloaded\n",
                    self.torrent.get_name(),
                    idx
                ),
            ))
            .is_err()
        {
            println!("Failed to log a new downloaded piece");
        }
    }

    /// Logs when the file is already downloaded.
    fn log_downloaded_file(&self) {
        if self
            .tx_logger
            .send(MsgCoder::generate_message(
                GENERIC_LOG_TYPE,
                CLIENT_MODE_LOG,
                format!(
                    "Torrent: {} - File has been downloaded completely.\n",
                    self.torrent.get_name()
                ),
            ))
            .is_err()
        {
            println!("Failed to log file download");
        }
    }

    fn merge_pieces(&self) -> Result<(), String> {
        self.log_downloaded_file();

        PieceMerger::merge_pieces(
            &self.torrent.get_name(),
            &self.settings.get_downloads_dir(),
            self.torrent.get_n_pieces(),
        )
    }

    fn join_peer_conn_threads(&self, vec_threads: Vec<JoinHandle<()>>) -> Result<(), ClientError> {
        for thread in vec_threads {
            if thread.join().is_err() {
                return Err(ClientError::JoiningThreadsError);
            }
        }
        Ok(())
    }

    pub fn get_torrent_info(&self) -> TorrentInfo {
        self.torrent.clone()
    }

    pub fn get_dl_pieces(&self) -> Arc<RwLock<PieceBitfield>> {
        self.downloaded_pieces.clone()
    }

    pub fn get_peer_id(&self) -> Vec<u8> {
        self.our_id.clone()
    }

    pub fn get_download_dir(&self) -> String {
        self.settings.get_downloads_dir()
    }

    pub fn get_port(&self) -> String {
        self.settings.get_tcp_port()
    }
}

// Testing:
//  - Client creation
//  - Request to tracker
//  - Get peer list
//  - Get one peer and connect to it
//  - Download a valid piece

// (Test de cliente lo dejamos comentado porque puede tardar bastante tiempo en conectarse a un peer)
/*
#[cfg(test)]
mod tests {
    use glib::{MainContext, PRIORITY_DEFAULT};

    use crate::piece::Piece;
    use crate::{settings::Settings, torrent_finder::TorrentFinder};
    use std::sync::mpsc::channel;

    use super::*;

    #[test]
    fn integration_test_download_piece() {
        let torrent_path =
            "files_for_testing/torrents_testing/debian-11.3.0-amd64-netinst.iso.torrent";
        let settings = Arc::new(
            Settings::new("files_for_testing/settings_files_testing/settings.txt").handle_error(),
        );
        let (tx_logger, _rx) = channel();
        let (tx, rx) = channel();
        let (tx_gtk, _rx_gtk) = MainContext::channel(PRIORITY_DEFAULT);
        let _ = tx.send(tx_gtk.clone());
        let _ = tx.send(tx_gtk);
        let sh_rx = Arc::new(Mutex::new(rx));

        if let Ok(vec) = TorrentFinder::find(torrent_path, "files_for_testing/downloaded_files2", sh_rx.clone()) {
            let mut piece = Piece::new(0, vec[0].0.get_piece_length(), vec[0].0.get_hash(0));
            let piece_queue = Arc::new(RwLock::new(PieceQueue::new(&vec[0].0, &vec[0].1)));
            let mut client = Client::new(
                settings,
                vec[0].0.clone(),
                vec[0].1.clone(),
                tx_logger,
                sh_rx.clone(),
            );
            let (tx_peer_conn_to_client, _rx) = channel();

            if let Ok(response) = client.connect_to_tracker() {
                if let Ok(mut peer_list) = client.get_peer_list(&response) {
                    loop {
                        if let Some(new_peer) = peer_list.pop() {
                            let peer_conn = PeerConnection::new(
                                client.clone(),
                                new_peer,
                                piece_queue.clone(),
                                Sender::clone(&tx_peer_conn_to_client),
                            );
                            if let Ok(mut peer_connection) = peer_conn {
                                if peer_connection.exchange_handshake().is_ok() {
                                    if peer_connection.download_piece(&mut piece).is_ok() {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    assert!(piece.piece_is_valid());
                    return;
                }
            }
        }
        assert!(false);
    }
}
*/
