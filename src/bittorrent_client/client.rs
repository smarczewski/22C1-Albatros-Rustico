use super::piece_queue::PieceQueue;
use crate::bencode_type::BencodeType;
use crate::bitfield::PieceBitfield;
use crate::bittorrent_client::peer::Peer;
use crate::bittorrent_client::peer_connection::PeerConnection;
use crate::bittorrent_client::torrent_info::TorrentInfo;
use crate::bittorrent_client::tracker_request::TrackerRequest;
use crate::channel_msg_log::msg_coder::MsgCoder;
use crate::constants::*;
use crate::errors::*;
use crate::event_messages::NewEvent;
use crate::piece_merger::PieceMerger;
use crate::settings::Settings;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;
use std::thread::JoinHandle;

/// # struct PeerConnection
/// Represents the BitTorrent client.
#[derive(Debug, Clone)]
pub struct Client {
    settings: Arc<Settings>,
    peer_id: Vec<u8>,
    torrent: TorrentInfo,
    connection_counter: u32,
    downloaded_pieces: Arc<RwLock<PieceBitfield>>,
    tx_logger: Sender<String>,
}

impl Client {
    pub fn init(
        settings: Arc<Settings>,
        torrent: (TorrentInfo, PieceBitfield),
        tx_sv: Sender<NewEvent>,
        tx_logger: Sender<String>,
    ) {
        let mut client = Client::new(settings, torrent.0, torrent.1, tx_logger);
        if let Err(error) = client.run_client(tx_sv) {
            error.print_error();
        }
    }

    /// Receives the settings and the path of the torrent file.
    /// On success, returns a client which is correctly initialized
    pub fn new(
        settings: Arc<Settings>,
        torrent: TorrentInfo,
        bitfield: PieceBitfield,
        tx_logger: Sender<String>,
    ) -> Client {
        Client {
            settings,
            peer_id: PEER_ID.as_bytes().to_vec(),
            torrent,
            connection_counter: 0,
            downloaded_pieces: Arc::new(RwLock::new(bitfield)),
            tx_logger,
        }
    }

    /// The client runs. It implies:
    ///     - Client connects to tracker, sends the request and gets the tracker response.
    ///     - Gets peer list
    ///     - Chooses one peer, then connects to it
    ///     - A PeerConnection is created
    ///     - Downloading starts.
    ///
    /// On error, the client chooses another peer and tries downloading the piece again.
    pub fn run_client(&mut self, _tx: Sender<NewEvent>) -> Result<(), ClientError> {
        let response = self.connect_to_tracker()?;
        let peer_list = self.get_peer_list(&response)?;

        let mut vec_threads: Vec<JoinHandle<()>> = vec![];
        // let client_shared = Arc::new(self.clone());
        let (tx, rx) = mpsc::channel();
        let piece_queue = Arc::new(RwLock::new(PieceQueue::new(&self.torrent)));
        let download_finished = Arc::new(RwLock::new(false));

        for peer in peer_list {
            let thread = self.handle_connection(
                self.clone(),
                peer,
                Sender::clone(&tx),
                piece_queue.clone(),
                download_finished.clone(),
            );
            vec_threads.push(thread);
        }
        self.listen_for_new_events(rx, download_finished.clone());

        for thread in vec_threads {
            thread.join().expect("Error during threads joining");
        }
        if *download_finished.read().unwrap() && self.merge_pieces().is_ok() {
            return Ok(());
        }
        Err(ClientError::DownloadError)
    }

    /// The client connects to tracker, sends the request and receives the response.
    /// On success, returns the response.
    /// Otherwise, return an error.
    fn connect_to_tracker(&mut self) -> Result<BencodeType, ClientError> {
        let request = TrackerRequest::new(self);
        match request.make_request() {
            Ok(response) => {
                println!(
                    "\nConnected to the tracker. The response has been obtained successfully :)"
                );
                self.log_tracker_connection();
                Ok(response)
            }
            Err(_error) => Err(ClientError::TrackerConnectionError),
        }
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

            let ip_aux: [u8; 4] = peers_compact[idx..idx + 4].try_into().unwrap();
            let ip = format!("{}.{}.{}.{}", ip_aux[0], ip_aux[1], ip_aux[2], ip_aux[3]);

            let port = u16::from_be_bytes(peers_compact[idx + 4..idx + 6].try_into().unwrap());
            idx += 6;

            let mut peer = HashMap::new();
            let peer_ip = BencodeType::String(ip.into_bytes());
            let peer_port = BencodeType::Integer(port as i64);

            peer.insert("ip".to_string(), peer_ip);
            peer.insert("port".to_string(), peer_port);

            list.push(BencodeType::Dictionary(peer));
        }
    }

    fn handle_connection(
        &self,
        client: Client,
        peer: Peer,
        tx: Sender<NewEvent>,
        piece_queue: Arc<RwLock<PieceQueue>>,
        download_finished: Arc<RwLock<bool>>,
    ) -> JoinHandle<()> {
        let dl_pieces = self.downloaded_pieces.clone();

        thread::spawn(move || {
            let peer_connection =
                PeerConnection::new(client, peer, piece_queue.clone(), Sender::clone(&tx));

            if let Ok(mut new_peer_connection) = peer_connection {
                new_peer_connection.start_download(dl_pieces, download_finished);
            }
        })
    }

    fn listen_for_new_events(
        &mut self,
        rx: Receiver<NewEvent>,
        download_finished: Arc<RwLock<bool>>,
    ) {
        let mut piece_counter = 0;
        loop {
            println!("counter:{}", self.connection_counter);
            if let Ok(new_event_msg) = rx.recv() {
                match new_event_msg {
                    NewEvent::NewConnection(peer) => {
                        self.log_peer_connection(peer);
                        self.connection_counter += 1;
                    }
                    NewEvent::NewDownloadedPiece(piece) => {
                        self.downloaded_pieces
                            .write()
                            .unwrap()
                            .add_a_piece(piece.get_idx());
                        self.log_downloaded_piece(piece.get_idx());
                        piece_counter += 1;
                    }
                    NewEvent::ConnectionDropped => self.connection_counter -= 1,
                    _ => (),
                }
            }
            if self.connection_counter == 0 {
                break;
            } else if piece_counter == self.get_torrent_info().get_n_pieces() {
                *download_finished.write().unwrap() = true;
            }
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
                    "Connected to tracker: {} successfully",
                    self.torrent.get_announce()
                ),
            ))
            .is_err()
        {
            println!("Failed to log successful tracker connection");
        }
    }

    /// Logs peer connection.
    fn log_peer_connection(&self, peer: Peer) {
        if self
            .tx_logger
            .send(MsgCoder::generate_message(
                GENERIC_LOG_TYPE,
                CLIENT_MODE_LOG,
                format!(
                    "Torrent: {} - Connected to new peer - {}:{}",
                    self.torrent.get_name(),
                    peer.ip(),
                    peer.port()
                ),
            ))
            .is_err()
        {
            println!("Failed to log successful tracker connection");
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
                    "Torrent: {} - Piece #{} has been downloaded",
                    self.torrent.get_name(),
                    idx
                ),
            ))
            .is_err()
        {
            println!("Failed to log successful tracker connection");
        }
    }

    fn log_downloaded_file(&self) {
        if self
            .tx_logger
            .send(MsgCoder::generate_message(
                GENERIC_LOG_TYPE,
                CLIENT_MODE_LOG,
                format!(
                    "Torrent: {} - File has been downloaded completely.",
                    self.torrent.get_name()
                ),
            ))
            .is_err()
        {
            println!("Failed to log successful tracker connection");
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

    pub fn get_torrent_info(&self) -> TorrentInfo {
        self.torrent.clone()
    }

    pub fn get_peer_id(&self) -> Vec<u8> {
        self.peer_id.clone()
    }

    pub fn get_download_dir(&self) -> String {
        self.settings.get_downloads_dir()
    }

    pub fn get_port(&self) -> String {
        self.settings.get_tcp_port()
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
    // use crate::encoding_decoding::settings_parser::SettingsParser;

    // #[test]
    // fn client_is_created_correctly() {
    //     let settings = SettingsParser
    //         .parse_file("files_for_testing/settings_files_testing/valid_format_v2.txt")
    //         .unwrap();
    //     let client = Client::new(
    //         &settings,
    //         "files_for_testing/torrents_tracker_request/ubuntu-20.04.4-desktop-amd64.iso.torrent"
    //             .to_string(),
    //     );
    //     assert!(client.is_ok());
    // }
}
