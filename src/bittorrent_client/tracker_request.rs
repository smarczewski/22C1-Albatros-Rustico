use crate::bencode_type::BencodeType;
use crate::bittorrent_client::client::Client;
use crate::constants::*;
use crate::encoding_decoding::bencode_parser::BencodeParser;
use crate::encoding_decoding::encoder::Encoder;
use crate::errors::*;

use native_tls::TlsConnector;
use std::io::{BufRead, Cursor, Read, Write};
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
    uploaded: i64,
    downloaded: i64,
    left: i64,
    event: String,
}

impl TrackerRequest {
    // Creates a new request from a decoded torrent file and a client.
    // On success, returns the request.
    //Otherwise, returns error.
    pub fn new(torrent: BencodeType, client: &Client) -> Result<TrackerRequest, RequestError> {
        let url_aux = torrent
            .get_value_from_dict("announce")
            .map_err(RequestError::TorrentInInvalidFormat)?
            .get_string()
            .map_err(RequestError::TorrentInInvalidFormat)?;

        let length = torrent
            .get_value_from_dict("info")
            .map_err(RequestError::TorrentInInvalidFormat)?
            .get_value_from_dict("length")
            .map_err(RequestError::TorrentInInvalidFormat)?
            .get_integer()
            .map_err(RequestError::TorrentInInvalidFormat)?;

        Ok(TrackerRequest {
            url: String::from_utf8(url_aux).map_err(RequestError::StrConvertionError)?,
            info_hash: client.info_hash(),
            peer_id: client.peer_id(),
            port: client.port(),
            uploaded: 0,
            downloaded: 0,
            left: length,
            event: "started".to_string(),
        })
    }

    /// Sends the http request to the tracker.
    /// On success, returns the tracker response (decoded).
    /// Otherwise, returns error
    pub fn send_request(&self) -> Result<BencodeType, RequestError> {
        let domain = self.get_domain(&self.url)?;
        let address = domain.clone() + ":" + TRACKER_PORT;

        let connector = TlsConnector::new().map_err(RequestError::TlsConnectionError)?;
        let tcp_stream = TcpStream::connect(&address).unwrap();
        let mut stream = connector.connect(&domain, tcp_stream).unwrap();

        let info_hash = Encoder.urlencode(self.info_hash.as_slice());
        let peer_id = Encoder.urlencode(self.peer_id.as_slice());

        let params = format!(
            "{}?info_hash={}&peer_id={}&port={}&uploaded={}&downloaded={}&left={}&event={}",
            self.url,
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

    fn get_domain(&self, url: &str) -> Result<String, RequestError> {
        if url.contains("https://") && url.contains("/announce") {
            return Ok(url.replace("https://", "").replace("/announce", ""));
        } else if url.contains("http://") && url.contains("/announce") {
            return Ok(url.replace("http://", "").replace("/announce", ""));
        }
        Err(RequestError::InvalidURL)
    }

    fn get_response(&self, response: Vec<u8>) -> Result<BencodeType, RequestError> {
        let mut respone_lines = vec![];
        let lines = Cursor::new(&response);

        lines.split(b'\n').for_each(|line| {
            if let Ok(l) = line {
                respone_lines.push(l);
            }
        });

        // last line contains the information we need
        if let Some(r) = respone_lines.pop() {
            let parsed_res = BencodeParser.parse_vec(r);
            if let Ok(v) = parsed_res {
                return Ok(v);
            }
        }
        Err(RequestError::CannotGetResponse)
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
