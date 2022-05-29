use crate::bencode_type::BencodeType;
use crate::bittorrent_client::tracker_request::TrackerRequest;
use crate::constants::*;
use crate::encoding_decoding::bencode_parser::BencodeParser;
use crate::encoding_decoding::encoder::Encoder;
use crate::errors::*;

use sha1::{Digest, Sha1};

use std::collections::HashMap;
use std::io::Error;
use std::thread;

pub struct Client {
    _logs_dir_path: String,
    _download_dir_path: String,
    tcp_port: String,
    torrent_path: String,
    peer_id: Vec<u8>,
    info_hash: Vec<u8>,
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
            let mut client = Client {
                _logs_dir_path: l.clone(),
                _download_dir_path: d.clone(),
                tcp_port: p.clone(),
                torrent_path: torrent_path.clone(),
                peer_id: PEER_ID.as_bytes().to_vec(),
                info_hash: [0u8; 20].to_vec(),
            };
            client.get_info_hash(&torrent_path)?;
            return Ok(client);
        }
        Err(ClientError::InvalidSettings)
    }

    pub fn run_client(&mut self) -> Result<(), Error> {
        let response = self.send_tracker_request().unwrap();
        println!("{:?}", response);
        let a_connection = thread::spawn(move || {
            // connect to peer
            //if let Ok(mut s) = stream {
            // do stuff
            //
        });

        a_connection.join().unwrap();
        Ok(())
    }

    pub fn info_hash(&self) -> Vec<u8> {
        self.info_hash.clone()
    }

    pub fn peer_id(&self) -> Vec<u8> {
        self.peer_id.clone()
    }

    pub fn port(&self) -> String {
        self.tcp_port.clone()
    }

    fn send_tracker_request(&mut self) -> Result<BencodeType, RequestError> {
        let benc_torrent = BencodeParser
            .parse_file(&self.torrent_path)
            .map_err(RequestError::ParserError)?;

        let request = TrackerRequest::new(benc_torrent, self)?;
        request.send_request()
    }

    fn get_info_hash(&mut self, torrent_path: &str) -> Result<(), ClientError> {
        let benc_torrent = BencodeParser
            .parse_file(torrent_path)
            .map_err(ClientError::NoSuchTorrentFile)?;

        let info_value = benc_torrent
            .get_value_from_dict("info")
            .map_err(ClientError::TorrentInInvalidFormat)?;
        let benc_info_value = Encoder.bencode(&info_value);

        let mut hasher = Sha1::new();
        hasher.update(benc_info_value);
        let result = hasher.finalize();

        self.info_hash = result.to_vec();
        Ok(())
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
