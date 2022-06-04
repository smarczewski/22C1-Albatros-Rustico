use crate::bencode_type::BencodeType;
use crate::bittorrent_client::client::Client;
use crate::constants::*;
use crate::encoding_decoding::bencode_parser::BencodeParser;
use crate::encoding_decoding::encoder::Encoder;
use crate::errors::*;

use native_tls::TlsConnector;
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
    uploaded: u32,
    downloaded: u32,
    left: u32,
    event: String,
}

impl TrackerRequest {
    // Creates a new request from a decoded torrent file and a client.
    // On success, returns the request.
    //Otherwise, returns error.
    pub fn new(client: &Client) -> TrackerRequest {
        let torrent_info = client.get_torrent_info();

        TrackerRequest {
            url: torrent_info.get_announce(),
            info_hash: torrent_info.get_info_hash(),
            peer_id: client.get_peer_id(),
            port: client.get_port(),
            uploaded: 0,
            downloaded: 0,
            left: torrent_info.get_length(),
            event: "started".to_string(),
        }
    }

    /// Sends the http request to the tracker.
    /// On success, returns the tracker response (decoded).
    /// Otherwise, returns error
    pub fn send_request(&self) -> Result<BencodeType, RequestError> {
        let domain = self.get_domain(&self.url);
        let address = domain.clone() + ":" + TRACKER_PORT;

        let connector = TlsConnector::new().map_err(RequestError::TlsConnectionError)?;
        let tcp_stream = TcpStream::connect(&address).unwrap();
        let mut stream = connector.connect(&domain, tcp_stream).unwrap();

        let info_hash = Encoder.urlencode(self.info_hash.as_slice());
        let peer_id = Encoder.urlencode(self.peer_id.as_slice());

        let params = format!(
            "https://{}/announce?info_hash={}&peer_id={}&port={}&uploaded={}&downloaded={}&left={}&event={}",
            domain,
            info_hash,
            peer_id,
            self.port,
            self.uploaded,
            self.downloaded,
            self.left,
            self.event
        );
        let req = format!("GET {} HTTP/1.0\r\n\r\n", params);

        stream.write_all(req.as_bytes()).unwrap();
        let mut res = vec![];
        stream.read_to_end(&mut res).unwrap();
        self.get_response(res)
    }

    fn get_domain(&self, url: &str) -> String {
        let url_aux = url
            .replace("http://", "")
            .replace("https://", "")
            .replace("/announce", "");
        if url_aux.contains(':') {
            if let Some((domain, _port)) = url_aux.split_once(':') {
                return domain.to_string();
            }
        }
        url_aux
    }

    fn get_response(&self, response_aux: Vec<u8>) -> Result<BencodeType, RequestError> {
        // We need to skip the first 9 lines because the response contains
        // information that we don't need (Request code, Date, etc.)
        let mut new_line_counter = 0;
        let mut idx = 0;
        while new_line_counter < 9 {
            if response_aux[idx] == (b'\n') {
                new_line_counter += 1;
            }
            idx += 1;
        }
        let (_, response) = response_aux.split_at(idx);
        // Then, we have to parse the response
        let parsed_res = BencodeParser.parse_vec(response);
        match parsed_res {
            Ok(r) => Ok(r),
            Err(_) => Err(RequestError::CannotGetResponse),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoding_decoding::settings_parser::SettingsParser;
    use crate::errors::ClientError;
    use std::collections::HashMap;

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
        let settings = SettingsParser
            .parse_file("files_for_testing/settings_files_testing/valid_format_v1.txt")
            .unwrap();
        let client = Client::new(&settings, path.to_string())?;

        Ok(client)
    }

    #[test]
    fn check_request_creation() {
        let torrent_path =
            "files_for_testing/torrents_tracker_request/ubuntu-20.04.4-desktop-amd64.iso.torrent";
        let benc_torrent = BencodeParser.parse_file(&torrent_path).unwrap();
        let client = create_client(&torrent_path).unwrap();
        let request = TrackerRequest::new(benc_torrent, &client).unwrap();

        let expected_req = TrackerRequest {
            url: "https://torrent.ubuntu.com/announce".to_string(),
            info_hash: urldecode("%F0%9C%8D%08%84Y%00%88%F4%00N%01%0A%92%8F%8Bax%C2%FD"),
            peer_id: PEER_ID.as_bytes().to_vec(),
            port: "6881".to_string(),
            uploaded: 0,
            downloaded: 0,
            left: 3379068928,
            event: "started".to_string(),
        };

        assert_eq!(request, expected_req);
    }

    #[test]
    fn request_creation_error() {
        let torrent_path =
            "files_for_testing/torrents_tracker_request/ubuntu-20.04.4-desktop-amd64.iso.torrent";
        let benc_torrent = BencodeType::Dictionary(HashMap::new());
        let client = create_client(&torrent_path).unwrap();
        let request = TrackerRequest::new(benc_torrent, &client);

        match request {
            Err(RequestError::TorrentInInvalidFormat(_)) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn send_request() {
        let torrent_path =
            "files_for_testing/torrents_tracker_request/ubuntu-20.04.4-desktop-amd64.iso.torrent";
        let benc_torrent = BencodeParser.parse_file(&torrent_path).unwrap();
        let client = create_client(&torrent_path).unwrap();
        let request = TrackerRequest::new(benc_torrent, &client).unwrap();
        let response = request.send_request().unwrap();

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
    }

    #[test]
    fn error_send_request_invalid_url() {
        let torrent_path = "files_for_testing/torrents_tracker_request/invalid_url.torrent";
        let benc_torrent = BencodeParser.parse_file(&torrent_path).unwrap();
        let client = create_client(&torrent_path).unwrap();
        let request = TrackerRequest::new(benc_torrent, &client).unwrap();
        let response = request.send_request();

        match response {
            Err(RequestError::InvalidURL) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn error_send_request_invalid_info() {
        let torrent_path = "files_for_testing/torrents_tracker_request/invalid_info.torrent";
        let benc_torrent = BencodeParser.parse_file(&torrent_path).unwrap();
        let client = create_client(&torrent_path).unwrap();
        let request = TrackerRequest::new(benc_torrent, &client).unwrap();
        let response = request.send_request().unwrap();

        let x = response.get_value_from_dict("failure reason");
        match x {
            Ok(_) => assert!(true),
            _ => assert!(false),
        }
    }
}
