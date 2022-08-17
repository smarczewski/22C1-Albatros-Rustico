use std::{
    fs::{self, File},
    net::{TcpListener, TcpStream},
    path::Path,
};

use crate::{
    constants::{DATA_DIR, THREADPOOL_SIZE},
    data::{hosted_peer::HostedPeer, tracker_data::TrackerData},
    errors::TrackerError,
    http_request::{Event, HttpRequest},
    threadpool::ThreadPool,
};

/// # BitTorrent Tracker
/// Represents a BitTorrent tracker, which will listen for requests and handle them.
/// This tracker can handle:
///     - Announce
///     - Stats
pub struct Tracker {
    listener: TcpListener,
}

impl Tracker {
    /// Returns an initialized tracker
    pub fn new(address: &str) -> Result<Tracker, TrackerError> {
        if let Ok(listener) = TcpListener::bind(address) {
            return Ok(Tracker { listener });
        };
        Err(TrackerError::InvalidAddress)
    }

    /// Runs the tracker. Tracker starts listening for new connections and then handles them
    pub fn run(&self) {
        let pool = ThreadPool::new(THREADPOOL_SIZE);

        for stream in self.listener.incoming().flatten() {
            println!("New connection!");
            pool.execute(|| {
                Tracker::handle_connection(stream);
            });
        }
    }

    /// Handles the connection:
    ///     - Parses the request
    ///     - Updates the stored data
    ///     - Responds to the request
    ///     - Adds the new peer to the stored data
    fn handle_connection(mut stream: TcpStream) {
        if !Path::new(DATA_DIR).exists() && File::create(DATA_DIR).is_err() {
            println!("Cannot create the file to store the tracker data");
        }

        let request = HttpRequest::new(&mut stream);
        Tracker::update_data();
        request.respond(&mut stream);

        if let Ok(ip_addr) = stream.peer_addr() {
            Tracker::add_new_peer(&request, ip_addr.ip().to_string());
        }
    }

    /// Updates the data marking peers as disconnected or removing them if necessary.
    fn update_data() {
        if let Ok(data_string) = fs::read_to_string(DATA_DIR) {
            let data_struct: Result<TrackerData, serde_json::Error> =
                serde_json::from_str(&data_string);

            if let Ok(mut data) = data_struct {
                data.update();

                if let Ok(serialized) = serde_json::to_string(&data) {
                    let _ = fs::write(DATA_DIR, serialized);
                }
            }
        }
    }

    /// Adds new peer to the stored data
    fn add_new_peer(request: &HttpRequest, ip_addr: String) {
        if let HttpRequest::Announce(announce) = request {
            let info_hash = announce.get_info_hash();
            let peer_id = announce.get_peer_id();
            let port = announce.get_port();
            let left = announce.get_left();
            let event = announce.get_event();

            let peer = HostedPeer::new(&peer_id, &ip_addr, &port, event, left);

            if let Ok(data_string) = fs::read_to_string(DATA_DIR) {
                let data_struct: Result<TrackerData, serde_json::Error> =
                    serde_json::from_str(&data_string);

                let mut tracker_data = match data_struct {
                    Ok(data) => data,
                    Err(_) => TrackerData::new(),
                };

                if let Event::Stopped = announce.get_event() {
                    tracker_data.remove_peer(&info_hash, peer);
                } else {
                    tracker_data.add_torrent(&info_hash, peer);
                }

                if let Ok(serialized) = serde_json::to_string(&tracker_data) {
                    let _ = fs::write(DATA_DIR, serialized);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cannot_create_create_tracker_with_invalid_address() {
        let address = "99999.0.0.1:999999999999";
        let tracker = Tracker::new(address);

        assert!(tracker.is_err());
    }
}
