use crate::bencode_type::BencodeType;
use crate::errors::ClientError;

pub struct Peer {
    id: Vec<u8>,
    ip: String,
    port: i64,
}

impl Peer {
    pub fn new(peer_list: &mut Vec<BencodeType>) -> Result<Peer, ClientError> {
        let last_peer = peer_list.pop();
        if let Some(peer) = last_peer {
            let peer_id = get_peer_id(&peer);
            let peer_ip = get_peer_ip(&peer);
            let peer_port = get_peer_port(&peer);

            if let (Ok(id), Ok(ip), Ok(port)) = (peer_id, peer_ip, peer_port) {
                return Ok(Peer { id, ip, port });
            }
        }

        Err(ClientError::InvalidTrackerResponse)
        //Return error + handle error + print
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
    // Return error+ Handle Error + print
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
    // Return error+ Handle Error + print
}

fn get_peer_port(peer: &BencodeType) -> Result<i64, ClientError> {
    if let Ok(value1) = peer.get_value_from_dict("port") {
        if let Ok(value2) = value1.get_integer() {
            return Ok(value2);
        }
    }
    // Return error+ Handle Error + print
    Err(ClientError::InvalidTrackerResponse)
}
