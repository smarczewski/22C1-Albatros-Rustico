use super::piece_queue::PieceQueue;
use crate::bencode_type::BencodeType;
use crate::bittorrent_client::peer::Peer;
use crate::bittorrent_client::torrent_info::TorrentInformation;
use crate::bittorrent_client::tracker_request::TrackerRequest;
use crate::channel_msg_log::msg_coder::MsgCoder;
use crate::constants::*;
use crate::errors::*;
use crate::peer_connection::PeerConnection;
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;
use std::thread::JoinHandle;

/// # struct PeerConnection
/// Represents the BitTorrent client.
pub struct Client {
    download_dir_path: String,
    tcp_port: String,
    peer_id: Vec<u8>,
    torrent_info: TorrentInformation,
    piece_queue: Arc<RwLock<PieceQueue>>,
}

impl Client {
    /// Receives the settings HashMap and the path of the torrent file.
    /// On success, returns a client which is correctly initialized
    pub fn new(
        settings: &HashMap<String, String>,
        torrent_path: String,
    ) -> Result<Client, ClientError> {
        if torrent_path.is_empty() {
            return Err(ClientError::EmptyTorrentPath);
        }

        let download = settings.get(&"download_dir_path".to_string());
        let port = settings.get(&"tcp_port".to_string());
        let torrent_info = TorrentInformation::new(&torrent_path)?;
        let torrent_info_ = TorrentInformation::new(&torrent_path)?; // RE-PENSAR
        let piece_queue = PieceQueue::new(torrent_info_)?;

        if let (Some(d), Some(p)) = (download, port) {
            return Ok(Client {
                download_dir_path: d.clone(),
                tcp_port: p.clone(),
                peer_id: PEER_ID.as_bytes().to_vec(),
                torrent_info,
                piece_queue: Arc::new(RwLock::new(piece_queue)),
            });
        }
        Err(ClientError::InvalidSettings)
    }

    /// The client runs. It implies:
    ///     - Client connects to tracker, sends the request and gets the tracker response.
    ///     - Gets peer list
    ///     - Chooses one peer, then connects to it
    ///     - A PeerConnection is created
    ///     - Downloading starts.
    ///
    /// On error, the client chooses another peer and tries downloading the piece again.
    pub fn run_client(&mut self, _tx: Sender<String>) -> Result<(), ClientError> {
        let response = self.connect_to_tracker()?;
        // self.log_tracker_connection(&tx);
        let mut peer_list = self.get_peer_list(&response)?;
        let mut vec_threads: Vec<JoinHandle<()>> = vec![];

        for _ in 0..peer_list.len() {
            let peer = Peer::new(&mut peer_list)?;
            // clonar todo lo que pasamos por parametro
            let info_hash = self.torrent_info.get_info_hash();
            let hashes_list = self.torrent_info.get_hashes_list();
            let torrent_name = self.torrent_info.get_name();
            let peer_id = self.get_peer_id();
            let piece_queue_clone = self.piece_queue.clone();
            let download_dir_path = self.download_dir_path.clone();

            let thread = thread::spawn(move || {
                // We create a new peer connection
                let peer_connection = PeerConnection::new(
                    download_dir_path,
                    torrent_name,
                    hashes_list,
                    peer,
                    piece_queue_clone,
                    info_hash,
                    peer_id,
                );

                match peer_connection {
                    Ok(mut new_peer_connection) => {
                        if let Err(e) = new_peer_connection.start_download() {
                            e.print_error()
                        }
                    }
                    Err(error) => error.print_error(),
                }
            });
            vec_threads.push(thread);
        }

        for t in vec_threads {
            t.join().unwrap();
        }
        Ok(())
    }

    /// The client connects to tracker, sends the request and receives the response.
    /// On success, returns the response.
    /// Otherwise, return an error.
    fn connect_to_tracker(&mut self) -> Result<BencodeType, ClientError> {
        let request = TrackerRequest::new(self);
        match request.send_request() {
            Ok(response) => {
                println!(
                    "\nConnected to the tracker. The response has been obtained successfully :)"
                );
                Ok(response)
            }
            Err(_error) => Err(ClientError::TrackerConnectionError),
        }
    }

    /// Gets the peer list from the tracker response.
    fn get_peer_list(&self, response: &BencodeType) -> Result<Vec<BencodeType>, ClientError> {
        if let Ok(peers_benc) = response.get_value_from_dict("peers") {
            if let Ok(peer_list) = peers_benc.get_list() {
                if !peer_list.is_empty() {
                    return Ok(peer_list);
                }
            }
        }
        println!("The tracker response is invalid. Cannot continue :(");
        Err(ClientError::InvalidTrackerResponse)
    }

    /// Logs tracker connection
    fn _log_tracker_connection(&self, tx: &Sender<String>) {
        if tx
            .send(MsgCoder::generate_message(
                GENERIC_LOG_TYPE,
                CLIENT_MODE_LOG,
                "Connected to tracker successfully".to_string(),
            ))
            .is_err()
        {
            println!("Failed to log successful tracker connection");
        }
    }

    /// Logs peer connection.
    fn _log_peer_connection(&self, tx: &Sender<String>, peer: &Peer) {
        let msg = format!(
            "Connected to peer {}:{} successfully.",
            peer.ip(),
            peer.port()
        );
        if tx
            .send(MsgCoder::generate_message(
                GENERIC_LOG_TYPE,
                CLIENT_MODE_LOG,
                msg,
            ))
            .is_err()
        {
            println!("Failed to log successful peer connection");
        }
    }

    /// Logs piece downloading.
    fn _log_downloaded_piece(&self, tx: &Sender<String>, idx: u32) {
        if tx
            .send(MsgCoder::generate_message(
                GENERIC_LOG_TYPE,
                CLIENT_MODE_LOG,
                format!("Piece {} has been successfully downloaded", idx),
            ))
            .is_err()
        {
            println!("Failed to log succesful piece downloading");
        }
    }

    pub fn get_torrent_info(&self) -> &TorrentInformation {
        &self.torrent_info
    }

    pub fn get_peer_id(&self) -> Vec<u8> {
        self.peer_id.clone()
    }

    pub fn get_port(&self) -> String {
        self.tcp_port.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoding_decoding::settings_parser::SettingsParser;

    #[test]
    fn client_is_created_correctly() {
        let settings = SettingsParser
            .parse_file("files_for_testing/settings_files_testing/valid_format_v2.txt")
            .unwrap();
        let client = Client::new(
            &settings,
            "files_for_testing/torrents_tracker_request/ubuntu-20.04.4-desktop-amd64.iso.torrent"
                .to_string(),
        );
        assert!(client.is_ok());
    }
}
