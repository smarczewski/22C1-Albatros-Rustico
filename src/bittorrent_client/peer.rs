use crate::bencode_type::BencodeType;
use crate::errors::ClientError;

/// # struct Peer
/// Represents a peer.
pub struct Peer {
    id: Vec<u8>,
    ip: String,
    port: i64,
}

impl Peer {
    /// Receives a list of peers and gets the information of the last peer.
    /// On success, returns a Peer.
    /// Otherwise, returns ClientError (CannotFindAnyPeer or InvalidTrackerResponse if
    /// the list is not valid)-
    pub fn new(peer_list: &mut Vec<BencodeType>) -> Result<Peer, ClientError> {
        let last_peer = peer_list.pop();
        if let Some(peer) = last_peer {
            let id = get_peer_id(&peer)?;
            let ip = get_peer_ip(&peer)?;
            let port = get_peer_port(&peer)?;

            return Ok(Peer { id, ip, port });
        }
        Err(ClientError::CannotFindAnyPeer)
    }

    pub fn id(&self) -> Vec<u8> {
        self.id.clone()
    }

    pub fn ip(&self) -> String {
        self.ip.clone()
    }

    pub fn port(&self) -> i64 {
        self.port
    }
}

fn get_peer_id(peer: &BencodeType) -> Result<Vec<u8>, ClientError> {
    if let Ok(value1) = peer.get_value_from_dict("peer id") {
        if let Ok(value2) = value1.get_string() {
            return Ok(value2);
        }
    }
    Err(ClientError::InvalidTrackerResponse)
}

fn get_peer_ip(peer: &BencodeType) -> Result<String, ClientError> {
    if let Ok(value1) = peer.get_value_from_dict("ip") {
        if let Ok(value2) = value1.get_string() {
            if let Ok(value3) = String::from_utf8(value2) {
                return Ok(value3);
            }
        }
    }
    Err(ClientError::InvalidTrackerResponse)
}

fn get_peer_port(peer: &BencodeType) -> Result<i64, ClientError> {
    if let Ok(value1) = peer.get_value_from_dict("port") {
        if let Ok(value2) = value1.get_integer() {
            return Ok(value2);
        }
    }
    Err(ClientError::InvalidTrackerResponse)
}
