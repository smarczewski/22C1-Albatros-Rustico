use crate::bencode_type::BencodeType;
use crate::bittorrent_client::client::Client;
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

    fn get_response(&self, response_aux: Vec<u8>) -> Result<BencodeType, RequestError> {
        // We need to skip the first lines because the response contains
        // information that we don't need (Request code, Date, etc.)
        let mut last_byte_1: u8 = response_aux[0];
        let mut last_byte_2: u8 = response_aux[1];
        let mut last_byte_3: u8 = response_aux[2];
        let mut idx = 3;

        // We want to split the vector at the begin of response.
        // So, we have to detect the next combination of bytes -> '\r','\n','\r','\n'
        while response_aux[idx] != (b'\n')
            || last_byte_1 != (b'\r')
            || last_byte_2 != (b'\n')
            || last_byte_3 != (b'\r')
        {
            if idx >= response_aux.len() {
                return Err(RequestError::CannotGetResponse);
            }
            last_byte_1 = last_byte_2;
            last_byte_2 = last_byte_3;
            last_byte_3 = response_aux[idx];
            idx += 1;
        }
        idx += 1;

        let (_, response) = response_aux.split_at(idx);
        // Then, we have to parse the response
        let parsed_res = BencodeParser.parse_vec(response);
        match parsed_res {
            Ok(r) => Ok(r),
            Err(_) => Err(RequestError::CannotGetResponse),
        }
    }
}
