use crate::{
    constants::DATA_DIR, data::tracker_data::TrackerData, encoding::encoder::Encoder,
    errors::TrackerError,
};
use std::{
    collections::HashMap,
    fs,
    io::{Read, Write},
    net::TcpStream,
};

#[derive(Debug, PartialEq)]
pub struct AnnounceEndpoint {
    info_hash: String,
    peer_id: String,
    port: u32,
    uploaded: u32,
    downloaded: u32,
    left: u32,
    event: Event,
}
/// # Announce Endpoint
/// Represents an Announce Request
impl AnnounceEndpoint {
    pub fn get_info_hash(&self) -> String {
        self.info_hash.clone()
    }

    pub fn get_peer_id(&self) -> String {
        self.peer_id.clone()
    }

    pub fn get_port(&self) -> u32 {
        self.port
    }

    pub fn get_left(&self) -> u32 {
        self.left
    }

    pub fn get_event(&self) -> Event {
        self.event.clone()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Started,
    Stopped,
    Completed,
    NotSpecified,
}

/// # Http Request
/// Represents a Http Request that supports this tracker
#[derive(Debug, PartialEq)]
pub enum HttpRequest {
    Announce(AnnounceEndpoint),
    Stats,
    JsFile(String),
    CssFile,
    Data,
    Unknown,
}

impl HttpRequest {
    /// Reads a request from a stream, then parses and returns it.
    pub fn new(stream: &mut TcpStream) -> Self {
        let mut buffer = [0; 1024];
        if stream.read(&mut buffer).is_ok() {
            let mut request = String::from_utf8_lossy(&buffer).to_string();
            println!("New Request:\n{}", request);
            if request.starts_with("GET /announce?") {
                let req_info = request.split_off("GET /announce?".len());
                if let Ok(http_req) = HttpRequest::parse_announce_req(req_info) {
                    return http_req;
                }
            } else if request.starts_with("GET /stats HTTP/1.1\r\n") {
                return HttpRequest::Stats;
            } else if request.starts_with("GET /styles.css HTTP/1.1\r\n") {
                return HttpRequest::CssFile;
            } else if request.starts_with("GET /script.js HTTP/1.1\r\n") {
                return HttpRequest::JsFile("page_files/script.js".to_string());
            } else if request.starts_with("GET /chartStyles.js HTTP/1.1\r\n") {
                return HttpRequest::JsFile("page_files/chartStyles.js".to_string());
            } else if request.starts_with("GET /data.json HTTP/1.1\r\n") {
                return HttpRequest::JsFile(DATA_DIR.to_string());
            }
        }
        HttpRequest::Unknown
    }

    /// Sends a response according to the type of request.
    pub fn respond(&self, stream: &mut TcpStream) {
        let (status_line, contents) = match self {
            HttpRequest::Announce(req) => HttpRequest::get_content_announce_req(req),
            HttpRequest::Stats => HttpRequest::get_content_stats_req(),
            HttpRequest::Unknown => HttpRequest::get_content_unknown_req(),
            HttpRequest::CssFile => HttpRequest::get_content_css(),
            HttpRequest::JsFile(file) => HttpRequest::get_content_js(file),
            HttpRequest::Data => HttpRequest::get_content_json(),
        };

        let response = format!("{}\r\n\r\n{}", status_line, contents);
        let _ = stream.write_all(response.as_bytes());
        _ = stream.flush();
    }

    /// Parses an Announce request
    fn parse_announce_req(request: String) -> Result<HttpRequest, TrackerError> {
        match request.split_once(' ') {
            Some((params, http_v)) => {
                if http_v.starts_with("HTTP/1.1\r\n") {
                    let param_dict = HttpRequest::parse_query_string(params)?;
                    let encoded_info_hash = param_dict
                        .get("info_hash")
                        .ok_or(TrackerError::InvalidRequest)?;
                    let decoded_info_hash = Encoder.urldecode(encoded_info_hash);
                    let peer_id = param_dict
                        .get("peer_id")
                        .ok_or(TrackerError::InvalidRequest)?;
                    let port = str::parse::<u32>(
                        param_dict.get("port").ok_or(TrackerError::InvalidRequest)?,
                    );
                    let uploaded = str::parse::<u32>(
                        param_dict
                            .get("uploaded")
                            .ok_or(TrackerError::InvalidRequest)?,
                    );
                    let downloaded = str::parse::<u32>(
                        param_dict
                            .get("downloaded")
                            .ok_or(TrackerError::InvalidRequest)?,
                    );
                    let left = str::parse::<u32>(
                        param_dict.get("left").ok_or(TrackerError::InvalidRequest)?,
                    );

                    let event = match param_dict.get("event") {
                        Some(&"started") => Event::Started,
                        Some(&"stopped") => Event::Stopped,
                        Some(&"completed") => Event::Completed,
                        None => Event::NotSpecified,
                        _ => return Err(TrackerError::InvalidRequest),
                    };

                    if let (Ok(info_hash), Ok(port_v), Ok(ul_v), Ok(dl_v), Ok(left_v)) =
                        (decoded_info_hash, port, uploaded, downloaded, left)
                    {
                        let announce_req = AnnounceEndpoint {
                            info_hash: info_hash.to_lowercase(),
                            peer_id: peer_id.to_string(),
                            port: port_v,
                            uploaded: ul_v,
                            downloaded: dl_v,
                            left: left_v,
                            event,
                        };
                        return Ok(HttpRequest::Announce(announce_req));
                    }
                }
                Err(TrackerError::InvalidRequest)
            }
            None => Err(TrackerError::InvalidRequest),
        }
    }

    fn parse_query_string(params: &str) -> Result<HashMap<&str, &str>, TrackerError> {
        let mut params_parsed = HashMap::new();
        for param in params.split('&') {
            match param.split_once('=') {
                Some((key, value)) => {
                    params_parsed.insert(key, value);
                }
                _ => return Err(TrackerError::InvalidRequest),
            }
        }

        Ok(params_parsed)
    }

    /// Returns the content of the response of an announce request and the status line.
    fn get_content_announce_req(req: &AnnounceEndpoint) -> (String, String) {
        let status_line = "HTTP/1.1 200 OK".to_string();
        if let Ok(data_string) = fs::read_to_string(DATA_DIR) {
            let data_struct: Result<TrackerData, serde_json::Error> =
                serde_json::from_str(&data_string);

            if let Ok(tracker_data) = data_struct {
                println!("{:?}; {}", tracker_data, req.get_info_hash());
                if let Ok(bencoded_data) = tracker_data.bencode_data(req.get_info_hash()) {
                    if let Ok(content) = String::from_utf8(bencoded_data) {
                        return (status_line, content);
                    }
                }
            }
        }
        (
            "HTTP/1.1 404 NOT FOUND".to_string(),
            "Sorry! Cannot find the requested torrent :(".to_string(),
        )
    }

    /// Returns the content of the response of an stats request (html file) and the status line.
    fn get_content_stats_req() -> (String, String) {
        let status_line = "HTTP/1.1 200 OK";
        let content_type = "Content-Type:text/html";
        let stats_file = "page_files/index.html";
        if let Ok(contents) = fs::read_to_string(stats_file) {
            let length = format!("Content-Length: {}", contents.len());

            return (
                format!("{}\r\n{}\r\n{}", status_line, length, content_type),
                contents,
            );
        }
        (
            "HTTP/1.1 404 NOT FOUND".to_string(),
            "Sorry! Cannot get the requested file :(".to_string(),
        )
    }

    /// Returns the content of the css file request and the status line
    fn get_content_css() -> (String, String) {
        let status_line = "HTTP/1.1 200 OK";
        let content_type = "Content-Type:text/css";
        let css_file = "page_files/styles.css";
        if let Ok(contents) = fs::read_to_string(css_file) {
            let length = format!("Content-Length: {}", contents.len());

            return (
                format!("{}\r\n{}\r\n{}", status_line, length, content_type),
                contents,
            );
        }
        (
            "HTTP/1.1 404 NOT FOUND".to_string(),
            "Sorry! Cannot get the requested file :(".to_string(),
        )
    }

    /// Returns the content of the javascript file request and the status line
    fn get_content_js(file: &str) -> (String, String) {
        let status_line = "HTTP/1.1 200 OK";
        let content_type = "Content-Type:text/js";
        if let Ok(contents) = fs::read_to_string(file) {
            let length = format!("Content-Length: {}", contents.len());

            return (
                format!("{}\r\n{}\r\n{}", status_line, length, content_type),
                contents,
            );
        }
        (
            "HTTP/1.1 404 NOT FOUND".to_string(),
            "Sorry! Cannot get the requested file :(".to_string(),
        )
    }

    /// Returns the content of the json file request and the status line
    fn get_content_json() -> (String, String) {
        let status_line = "HTTP/1.1 200 OK";
        let content_type = "Content-Type:text/json";
        let json_file = DATA_DIR;
        if let Ok(contents) = fs::read_to_string(json_file) {
            let length = format!("Content-Length: {}", contents.len());

            return (
                format!("{}\r\n{}\r\n{}", status_line, length, content_type),
                contents,
            );
        }
        (
            "HTTP/1.1 404 NOT FOUND".to_string(),
            "Sorry! Cannot get the requested file :(".to_string(),
        )
    }

    /// Returns the content of an unknown request and the status line
    fn get_content_unknown_req() -> (String, String) {
        let status_line = "HTTP/1.1 404 NOT FOUND".to_string();
        let stats_file = "page_files/404.html";
        if let Ok(contents) = fs::read_to_string(stats_file) {
            return (status_line, contents);
        }
        (
            "HTTP/1.1 404 NOT FOUND".to_string(),
            "Sorry! Cannot get the requested file :(".to_string(),
        )
    }
}

// si es announce -> chequear si hay que añadir y torrent al json
// implementar httprequest::response()

#[cfg(test)]
mod tests {
    use super::*;
    use std::{io::Write, net::TcpListener, thread};

    #[test]
    fn valid_request_stats() {
        let address = "127.0.0.1:8080";
        let listener = TcpListener::bind(address).unwrap();

        let cl_thread = thread::spawn(move || {
            if let Ok(mut stream_cl) = TcpStream::connect(address) {
                let _ = stream_cl.write(b"GET /stats HTTP/1.1\r\n");
            }
        });

        let (mut stream_sv, _socket_addr) = listener.accept().unwrap();
        let request = HttpRequest::new(&mut stream_sv);
        cl_thread.join().unwrap();

        assert_eq!(request, HttpRequest::Stats);
    }

    #[test]
    fn valid_request_announce() {
        let address = "127.0.0.1:8081";
        let listener = TcpListener::bind(address).unwrap();

        let cl_thread = thread::spawn(move || {
            if let Ok(mut stream_cl) = TcpStream::connect(address) {
                let announce = "GET /announce?info_hash=%f0%7e%0b%05%84%74%5b%7b%cb%35%e9%80%97%48%8d%34%e6%86%23%d0&peer_id=-AR1234-111111111111&port=6881&uploaded=0&downloaded=0&left=1502576640&event=started HTTP/1.1\r\n";
                let _ = stream_cl.write(announce.as_bytes());
            }
        });

        let (mut stream_sv, _socket_addr) = listener.accept().unwrap();
        let request = HttpRequest::new(&mut stream_sv);

        let exp_announce = AnnounceEndpoint {
            info_hash: "f07e0b0584745b7bcb35e98097488d34e68623d0".to_string(),
            peer_id: "-AR1234-111111111111".to_string(),
            port: 6881,
            uploaded: 0,
            downloaded: 0,
            left: 1502576640,
            event: Event::Started,
        };

        let exp_request = HttpRequest::Announce(exp_announce);
        cl_thread.join().unwrap();

        assert_eq!(request, exp_request);
    }

    #[test]
    fn invalid_request_announce() {
        let address = "127.0.0.1:8082";
        let listener = TcpListener::bind(address).unwrap();

        let cl_thread = thread::spawn(move || {
            if let Ok(mut stream_cl) = TcpStream::connect(address) {
                let announce = "GET /announce?peer_id=-AR1234-111111111111&port=6881&uploaded=0&downloaded=0&left=1502576640&event=started HTTP/1.1\r\n";
                let _ = stream_cl.write(announce.as_bytes());
            }
        });

        let (mut stream_sv, _socket_addr) = listener.accept().unwrap();
        let request = HttpRequest::new(&mut stream_sv);
        cl_thread.join().unwrap();

        assert_eq!(request, HttpRequest::Unknown);
    }

    #[test]
    fn invalid_request_announce2() {
        let address = "127.0.0.1:8094";
        let listener = TcpListener::bind(address).unwrap();

        let cl_thread = thread::spawn(move || {
            if let Ok(mut stream_cl) = TcpStream::connect(address) {
                let announce = "GET /announce?info_hash=%f0%7e%0b%05%84%74%5b%7b%cb%35%e9%80%97%48%8d%34%e6%86%23%d0&peer_id=-AR1234-111111111111&port=6881&uploaded=0&downloaded=0&left=1502576640&event=startedHTTP/1.1\r\n";
                let _ = stream_cl.write(announce.as_bytes());
            }
        });

        let (mut stream_sv, _socket_addr) = listener.accept().unwrap();
        let request = HttpRequest::new(&mut stream_sv);
        cl_thread.join().unwrap();

        assert_eq!(request, HttpRequest::Unknown);
    }

    #[test]
    fn invalid_request() {
        let address = "127.0.0.1:8095";
        let listener = TcpListener::bind(address).unwrap();

        let cl_thread = thread::spawn(move || {
            if let Ok(mut stream_cl) = TcpStream::connect(address) {
                let announce = "GET /unknown_endpoint HTTP/1.1\r\n";
                let _ = stream_cl.write(announce.as_bytes());
            }
        });

        let (mut stream_sv, _socket_addr) = listener.accept().unwrap();
        let request = HttpRequest::new(&mut stream_sv);
        cl_thread.join().unwrap();

        assert_eq!(request, HttpRequest::Unknown);
    }
}
