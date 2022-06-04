use crate::bencode_type::BencodeType;
use crate::bittorrent_client::peer::Peer;
use crate::bittorrent_client::torrent_info::TorrentInformation;
use crate::bittorrent_client::tracker_request::TrackerRequest;
use crate::constants::*;
use crate::errors::*;
use crate::peer_connection::PeerConnection;

use sha1::{Digest, Sha1};

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

pub struct Client {
    _logs_dir_path: String,
    download_dir_path: String,
    tcp_port: String,
    peer_id: Vec<u8>,
    torrent_info: TorrentInformation,
}

impl Client {
    pub fn new(
        settings: &HashMap<String, String>,
        torrent_path: String,
    ) -> Result<Client, ClientError> {
        if torrent_path.is_empty() {
            return Err(ClientError::EmptyTorrentPath);
        }

        let log = settings.get(&"logs_dir_path".to_string());
        let download = settings.get(&"download_dir_path".to_string());
        let port = settings.get(&"tcp_port".to_string());

        if let (Some(l), Some(d), Some(p)) = (log, download, port) {
            return Ok(Client {
                _logs_dir_path: l.clone(),
                download_dir_path: d.clone(),
                tcp_port: p.clone(),
                peer_id: PEER_ID.as_bytes().to_vec(),
                torrent_info: TorrentInformation::new(&torrent_path)?,
            });
        }
        Err(ClientError::InvalidSettings)
    }

    pub fn run_client(&mut self) -> Result<(), ClientError> {
        let response = self.connect_to_tracker()?;
        let mut peer_list = self.get_peer_list(&response)?;

        loop {
            let peer = Peer::new(&mut peer_list)?;
            if let Ok(mut peer_conn) = PeerConnection::new(self, peer) {
                println!("Connected to peer successfully!");
                if let Ok((idx, piece)) = peer_conn.download_piece() {
                    if self.piece_is_valid(idx, &piece)
                        && self.store_piece_in_file(idx, piece).is_ok()
                    {
                        break;
                    }
                } else {
                    println!("Downloaded piece is not valid");
                }
            } else {
                println!("Cannot connect to peer");
            }
        }
        Ok(())
    }

    fn connect_to_tracker(&mut self) -> Result<BencodeType, ClientError> {
        let request = TrackerRequest::new(self);
        match request.send_request() {
            Ok(response) => {
                println!(
                    "\nConnected to the tracker. The response has been obtained successfully :)"
                );
                Ok(response)
            }
            Err(error) => {
                println!("ERROR: {:?}", error);
                Err(ClientError::TrackerConnectionError)
            }
        }
    }

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

    fn piece_is_valid(&self, idx: u32, piece: &Vec<u8>) -> bool {
        //Get original hash
        let expected_hash = self.torrent_info.get_hash(idx);

        //Get hash of downloaded piece
        let mut hasher = Sha1::new();
        hasher.update(piece);
        let piece_hash = hasher.finalize();

        //Compare two hashes
        expected_hash == piece_hash.to_vec()
    }

    fn store_piece_in_file(&self, idx: u32, piece: Vec<u8>) -> Result<(), ClientError> {
        let path = format!(
            "{}/{}_piece_{}",
            self.download_dir_path,
            self.torrent_info.get_name(),
            idx
        );
        if let Ok(mut file) = File::create(path) {
            if file.write_all(&piece).is_ok() {
                return Ok(());
            }
        }
        println!("ERROR: Cannot store the piece in the file :(");
        Err(ClientError::StoringPieceError)
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
