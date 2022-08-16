use crate::bencode_type::BencodeType;
use crate::bt_client::client::Client;
use crate::constants::*;
use crate::encoding_decoding::bencode_parser::BencodeParser;
use crate::encoding_decoding::encoder::Encoder;
use crate::errors::*;

use native_tls::TlsConnector;
use native_tls::TlsStream;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::string::String;

/// # struct Tracker Request
/// Represents the HTTP Request that the client sends to the tracker
#[derive(Debug, PartialEq)]
pub struct TrackerRequest {
    url: String,
    info_hash: Vec<u8>,
    peer_id: Vec<u8>,
    port: String,
    uploaded: u64,
    downloaded: u64,
    left: u32,
    event: String,
}

impl TrackerRequest {
    /// Creates a new request from a decoded torrent file and a client.
    /// On success, returns the request.
    /// Otherwise, returns error.
    pub fn new(client: &Client, downloaded: u64) -> TrackerRequest {
        let torrent_info = client.get_torrent_info();

        TrackerRequest {
            url: torrent_info.get_announce(),
            info_hash: torrent_info.get_info_hash(),
            peer_id: client.get_peer_id(),
            port: client.get_port(),
            uploaded: 0,
            downloaded,
            left: torrent_info.get_length(),
            event: "started".to_string(),
        }
    }

    /// Sends the http request to the tracker.
    /// If the tracker url starts with "https", we use TlsStream, otherwise we use TcpStream.
    /// On success, returns the tracker response (decoded).
    /// Otherwise, returns error
    pub fn make_request(&self) -> Result<BencodeType, RequestError> {
        let (domain, port) = self.get_domain_and_port(&self.url);
        if self.url.contains("https://") {
            let mut stream = self.connect_to_https_tracker(&domain, HTTPS_TRACKER_PORT)?;
            self.send_request(&mut stream, domain)
        } else {
            let mut stream = self.connect_to_nonhttps_tracker(&domain, &port)?;
            self.send_request(&mut stream, domain)
        }
    }

    /// Connects to tracker whose url starts with "https"
    fn connect_to_https_tracker(
        &self,
        domain: &str,
        port: &str,
    ) -> Result<TlsStream<TcpStream>, RequestError> {
        let address = format!("{}:{}", domain, port);
        if let (Ok(connector), Ok(tcp_stream)) = (TlsConnector::new(), TcpStream::connect(&address))
        {
            if let Ok(stream) = connector.connect(domain, tcp_stream) {
                return Ok(stream);
            }
        }
        Err(RequestError::CannotConnectToTracker)
    }

    /// Connects to tracker whose url does not start with "https"
    fn connect_to_nonhttps_tracker(
        &self,
        domain: &str,
        port: &str,
    ) -> Result<TcpStream, RequestError> {
        let address = format!("{}:{}", domain, port);
        if let Ok(stream) = TcpStream::connect(address) {
            return Ok(stream);
        }
        Err(RequestError::CannotConnectToTracker)
    }

    fn get_domain_and_port(&self, url: &str) -> (String, String) {
        let url_aux = url
            .replace("udp://", "")
            .replace("http://", "")
            .replace("https://", "")
            .replace("/announce", "");
        if url_aux.contains(':') {
            if let Some((domain, port)) = url_aux.split_once(':') {
                return (domain.to_string(), port.to_string());
            }
        }
        (url_aux, "443".to_string())
    }

    fn send_request<T: Read + Write>(
        &self,
        stream: &mut T,
        domain: String,
    ) -> Result<BencodeType, RequestError> {
        let info_hash = Encoder.urlencode(self.info_hash.as_slice());
        let peer_id = Encoder.urlencode(self.peer_id.as_slice());

        let req = self.build_request(&domain, &info_hash, &peer_id);

        if stream.write_all(req.as_bytes()).is_ok() {
            let mut res = vec![];
            if stream.read_to_end(&mut res).is_ok() {
                return self.get_response(res);
            }
        }
        Err(RequestError::CannotGetResponse)
    }

    fn build_request(&self, domain: &str, info_hash: &str, peer_id: &str) -> String {
        let params = format!(
            "/announce?info_hash={}&peer_id={}&port={}&uploaded={}&downloaded={}&left={}&event={}",
            info_hash, peer_id, self.port, self.uploaded, self.downloaded, self.left, self.event
        );

        format!("GET {} HTTP/1.1\r\nHost: {}\r\n\r\n", params, domain)
    }

    fn get_response(&self, response: Vec<u8>) -> Result<BencodeType, RequestError> {
        // We need to skip the first lines because the response contains
        // information that we don't need (Request code, Date, etc.)
        let mut three_last_bytes = (response[0], response[1], response[0]);
        let mut idx = 3;

        // We want to split the vector at the begin of response.
        // So, we have to detect the next combination of bytes -> '\r','\n','\r','\n'
        while response[idx] != (b'\n')
            || three_last_bytes.0 != (b'\r')
            || three_last_bytes.1 != (b'\n')
            || three_last_bytes.2 != (b'\r')
        {
            if idx >= response.len() {
                return Err(RequestError::CannotGetResponse);
            }
            three_last_bytes.0 = three_last_bytes.1;
            three_last_bytes.1 = three_last_bytes.2;
            three_last_bytes.2 = response[idx];
            idx += 1;
        }
        idx += 1;

        let (_, response_split) = response.split_at(idx);
        // Then, we have to parse the response
        let parsed_res = BencodeParser.parse_vec(response_split);
        match parsed_res {
            Ok(r) => Ok(r),
            Err(_) => Err(RequestError::CannotGetResponse),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::channel;
    use std::sync::{Arc, Mutex, RwLock};

    use glib::PRIORITY_DEFAULT;

    use super::*;
    use crate::bitfield::PieceBitfield;
    use crate::errors::ClientError;
    use crate::settings::Settings;
    use crate::torrent_info::TorrentInfo;

    fn urldecode(data: &str) -> Vec<u8> {
        let mut decoded_data = Vec::<u8>::new();
        let chars: Vec<char> = data.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            if chars[i] == '%' {
                let mut x = String::new();
                x.push(chars[i + 1]);
                x.push(chars[i + 2]);
                i += 2;
                decoded_data.push(u8::from_str_radix(&x, 16).unwrap());
            } else {
                decoded_data.push(chars[i] as u8);
            }
            i += 1;
        }
        decoded_data
    }

    fn create_client(path: &str) -> Result<Client, ClientError> {
        let settings_path = "files_for_testing/settings_files_testing/valid_format_v1.txt";
        let settings = Settings::new(settings_path);
        let torrent = TorrentInfo::new(path);
        if let (Ok(s), Ok(t)) = (settings, torrent) {
            let dl_pieces = Arc::new(RwLock::new(PieceBitfield::new(t.get_n_pieces())));
            let (tx_logger, _rx) = channel();

            let (tx, rx) = channel();
            let (tx_gtk, _rx_gtk) = glib::MainContext::channel(PRIORITY_DEFAULT);
            let _ = tx.send(tx_gtk);

            let client = Client::new(
                Arc::new(s),
                t,
                dl_pieces,
                tx_logger,
                Arc::new(Mutex::new(rx)),
            );

            return Ok(client);
        }
        Err(ClientError::InvalidSettings)
    }

    #[test]
    fn check_request_creation() {
        let torrent_path =
            "files_for_testing/torrents_tracker_request_test/ubuntu-20.04.4-desktop-amd64.iso.torrent";
        if let Ok(client) = create_client(&torrent_path) {
            let request = TrackerRequest::new(&client, 0);

            let expected_req = TrackerRequest {
                url: "https://torrent.ubuntu.com/announce".to_string(),
                info_hash: urldecode("%F0%9C%8D%08%84Y%00%88%F4%00N%01%0A%92%8F%8Bax%C2%FD"),
                peer_id: CLIENT_ID.as_bytes().to_vec(),
                port: "8080".to_string(),
                uploaded: 0,
                downloaded: 0,
                left: 3379068928,
                event: "started".to_string(),
            };

            assert_eq!(request, expected_req);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn send_request() {
        let torrent_path =
            "files_for_testing/torrents_tracker_request_test/ubuntu-20.04.4-desktop-amd64.iso.torrent";
        if let Ok(client) = create_client(&torrent_path) {
            let request = TrackerRequest::new(&client, 0);
            if let Ok(response) = request.make_request() {
                let x1 = response.get_value_from_dict("interval");
                let x2 = response.get_value_from_dict("complete");
                let x3 = response.get_value_from_dict("incomplete");
                let x4 = response.get_value_from_dict("peers");

                match (x1, x2, x3, x4) {
                    (
                        Ok(BencodeType::Integer(_)),
                        Ok(BencodeType::Integer(_)),
                        Ok(BencodeType::Integer(_)),
                        Ok(BencodeType::List(_)),
                    ) => assert!(true),
                    _ => assert!(false),
                }
                return;
            }
        }
        assert!(false);
    }

    #[test]
    fn error_send_request_invalid_url() {
        let torrent_path = "files_for_testing/torrents_tracker_request_test/invalid_url.torrent";
        if let Ok(client) = create_client(&torrent_path) {
            let request = TrackerRequest::new(&client, 0);
            let response = request.make_request();

            match response {
                Err(RequestError::CannotConnectToTracker) => assert!(true),
                _ => assert!(false),
            }
        } else {
            assert!(false);
        }
    }

    #[test]
    fn error_send_request_invalid_info() {
        let torrent_path = "files_for_testing/torrents_tracker_request_test/invalid_info.torrent";
        if let Ok(client) = create_client(&torrent_path) {
            let request = TrackerRequest::new(&client, 0);
            if let Ok(response) = request.make_request() {
                let x = response.get_value_from_dict("failure reason");
                match x {
                    Ok(_) => assert!(true),
                    _ => assert!(false),
                }
                return;
            }
        }
        assert!(false);
    }
}
