use std::collections::HashMap;

use chrono::{DateTime, FixedOffset, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    constants::{A_WEEK_IN_SECS, THREE_DAYS_IN_SECS},
    encoding::bencode_type::BencodeType,
    http_request::Event,
};

/// # struct HostedPeer
/// Represents a peer hosted on the tracker and contains the following:
///     - peer_id
///     - peer_ip
///     - port
///     - dt_connection -> UTC time and date when the peer was added in RFC3339 format
///     - dt_disconnection -> UTC time and date when the peer was disconnected in RFC3339 format
///     - completed -> wether or not the peer has completed the download
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HostedPeer {
    peer_id: String,
    peer_ip: String,
    port: u32,
    dt_connection: String,
    dt_disconnection: Option<String>,
    completed: bool,
    dt_completion: Option<String>,
}

impl HostedPeer {
    pub fn new(peer_id: &str, peer_ip: &str, port: &u32, event: Event, left: u32) -> HostedPeer {
        let timestamp = Utc::now().to_rfc3339();

        let mut peer = HostedPeer {
            peer_id: peer_id.to_string(),
            peer_ip: peer_ip.to_string(),
            port: *port,
            dt_connection: timestamp,
            dt_disconnection: None,
            completed: event == Event::Completed || left == 0,
            dt_completion: None,
        };

        if peer.is_completed() {
            peer.change_to_completed();
        }

        peer
    }

    /// Returns the peer ID
    pub fn get_peer_id(&self) -> String {
        self.peer_id.clone()
    }

    /// Returns the peer IP address
    pub fn get_peer_ip(&self) -> String {
        self.peer_ip.clone()
    }

    /// Returns the peer port
    pub fn get_peer_port(&self) -> u32 {
        self.port
    }

    // Returns the timestamp in DateTime format
    pub fn get_timestamp(&self) -> DateTime<FixedOffset> {
        DateTime::parse_from_rfc3339(&self.dt_connection).unwrap()
    }

    /// Returns true if the peer has completed the download, false if it has not
    pub fn is_completed(&self) -> bool {
        self.completed
    }

    /// Returns true if the peer is still connected to tracker, false if it is not
    pub fn is_connected(&self) -> bool {
        self.dt_disconnection.is_none()
    }

    /// Changes the peer status to completed
    pub fn change_to_completed(&mut self) {
        self.completed = true;
        self.dt_completion = Some(Utc::now().to_rfc3339());
    }

    /// Checks if this peer has been connected for more than a week.
    pub fn has_to_be_disconnected(&mut self) -> bool {
        if let Ok(dt_connection) = DateTime::parse_from_rfc3339(&self.dt_connection) {
            if let Ok(curr_datetime) = DateTime::parse_from_rfc3339(&Utc::now().to_rfc3339()) {
                let ts_connection = dt_connection.timestamp();
                let ts_current = curr_datetime.timestamp();
                if ts_current - ts_connection >= A_WEEK_IN_SECS {
                    return true;
                }
            }
        }
        false
    }

    /// Marks this peer as disconnected setting its disconnection date
    pub fn set_as_disconnected(&mut self) {
        if let Ok(dt_connection) = DateTime::parse_from_rfc3339(&self.dt_connection) {
            let ts_connection = dt_connection.timestamp();

            let disconnection = NaiveDateTime::from_timestamp(ts_connection + A_WEEK_IN_SECS, 0);
            let dt_disconnection = DateTime::<Utc>::from_utc(disconnection, Utc);
            self.dt_disconnection = Some(dt_disconnection.to_rfc3339());
        }
    }

    /// Returns true if the peer should to be removed from the tracker, false if it should not
    pub fn has_to_be_removed(&self) -> bool {
        if let Some(disconnection) = &self.dt_disconnection {
            if let Ok(dt_disconnection) = DateTime::parse_from_rfc3339(disconnection) {
                if let Ok(curr_datetime) = DateTime::parse_from_rfc3339(&Utc::now().to_rfc3339()) {
                    let ts_disconnection = dt_disconnection.timestamp();
                    let ts_current = curr_datetime.timestamp();

                    if ts_current - ts_disconnection >= THREE_DAYS_IN_SECS {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Returns a bencoded dictionary that represents the peer
    pub fn to_bencode_type(&self) -> BencodeType {
        let mut peer_dict = HashMap::new();
        let ip = BencodeType::String(self.get_peer_ip().into_bytes());
        let port = BencodeType::Integer(self.port as i64);
        let id = BencodeType::String(self.get_peer_id().into_bytes());

        peer_dict.insert("ip".to_string(), ip);
        peer_dict.insert("port".to_string(), port);
        peer_dict.insert("peer id".to_string(), id);

        BencodeType::Dictionary(peer_dict)
    }
}
