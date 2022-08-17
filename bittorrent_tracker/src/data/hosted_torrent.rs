use std::{collections::HashMap, vec};

use crate::{data::hosted_peer::HostedPeer, encoding::bencode_type::BencodeType};
use serde::{Deserialize, Serialize};

/// # struct HostedTorrent
/// Represents a torrent hosted on the tracker and contains the following:
///     - info_hash
///     - timestamp -> UTC time and date when the torrent was added in RFC3339 format
///     - seeders -> number of peers with the entire file for this torrent
///     - leechers -> number of non-seeder peers
///     - peers -> vector containing the peers for this torrent
#[derive(Serialize, Deserialize, Debug)]
pub struct HostedTorrent {
    info_hash: String,
    timestamp: String,
    seeders: u32,
    leechers: u32,
    peers: Vec<HostedPeer>,
}

impl HostedTorrent {
    pub fn new(name: &str) -> HostedTorrent {
        let timestamp = chrono::Utc::now().to_rfc3339();
        let peers: Vec<HostedPeer> = vec![];

        HostedTorrent {
            info_hash: name.to_string(),
            timestamp,
            seeders: 0,
            leechers: 0,
            peers,
        }
    }

    /// Returns the infohash of the torrent
    pub fn get_infohash(&self) -> String {
        self.info_hash.clone()
    }

    /// Returns the amount of seeders (number of peers with the entire file)
    pub fn get_seeders(&self) -> u32 {
        self.seeders
    }

    /// Returns the amount of leechers (number of non-seeder peers)
    pub fn get_leechers(&self) -> u32 {
        self.leechers
    }

    /// Disconnects or removes a peer of this torrent if necessary
    pub fn update(&mut self) {
        let mut deleted_peers = vec![];

        for (idx, peer) in self.peers.iter_mut().enumerate() {
            if peer.has_to_be_disconnected() {
                peer.set_as_disconnected();
                if peer.is_completed() {
                    self.seeders -= 1;
                } else {
                    self.leechers -= 1;
                }
            }

            if peer.has_to_be_removed() {
                deleted_peers.push(idx);
            }
        }

        for idx in deleted_peers {
            self.peers.remove(idx);
        }
    }

    /// Adds a peer to the torrent
    pub fn add_peer(&mut self, peer: HostedPeer) {
        for p in &mut self.peers {
            // If the peer is already on our peers list
            if p.get_peer_ip() == peer.get_peer_ip() && p.get_peer_port() == peer.get_peer_port() {
                // And the request told us that it completed the download
                if peer.is_completed() {
                    // We increase our seeders counter for the torrent
                    // And change the peer's status on our list
                    self.seeders += 1;
                    self.leechers -= 1;
                    p.change_to_completed();
                }
                return;
            }
        }
        // If the peer wasn't on our list then we add it
        if peer.is_completed() {
            self.seeders += 1;
        } else {
            self.leechers += 1;
        }
        self.peers.push(peer);
    }

    pub fn remove_peer(&mut self, searched_peer: HostedPeer) {
        let mut idx_peer = None;

        for (idx, peer) in self.peers.iter_mut().enumerate() {
            if peer.get_peer_ip() == searched_peer.get_peer_ip()
                && peer.get_peer_port() == searched_peer.get_peer_port()
            {
                if peer.is_completed() {
                    self.seeders -= 1;
                } else {
                    self.leechers -= 1;
                }
                idx_peer = Some(idx);
                break;
            }
        }

        if let Some(idx) = idx_peer {
            self.peers.remove(idx);
        }
    }

    /// Returns a bencoded dictionary that represents the torrent
    pub fn to_bencode_type(&self) -> BencodeType {
        let mut data_dict = HashMap::new();
        let complete = BencodeType::Integer(self.seeders as i64);
        let incomplete = BencodeType::Integer(self.leechers as i64);

        let mut peer_list = vec![];
        for peer in &self.peers {
            if peer.is_connected() {
                let peer_bencoded = peer.to_bencode_type();
                peer_list.push(peer_bencoded);
            }
        }

        data_dict.insert("complete".to_string(), complete);
        data_dict.insert("incomplete".to_string(), incomplete);
        data_dict.insert("interval".to_string(), BencodeType::Integer(1800));
        data_dict.insert("peers".to_string(), BencodeType::List(peer_list));
        BencodeType::Dictionary(data_dict)
    }
}

#[cfg(test)]
mod tests {
    use crate::{data::hosted_peer::HostedPeer, http_request::Event};

    use super::HostedTorrent;

    #[test]
    fn adding_completed_peer_increases_seeders() {
        let mut torrent = HostedTorrent::new("f07e0b0584745b7bcb35e98097488d34e68623d0");
        let peer = HostedPeer::new(
            "-AR1234-111111111111",
            "127.0.0.1",
            &8080,
            Event::Completed,
            0,
        );
        torrent.add_peer(peer);
        assert_eq!(torrent.get_seeders(), 1);
    }

    #[test]
    fn adding_incompleted_peer_increases_leechers() {
        let mut torrent = HostedTorrent::new("f07e0b0584745b7bcb35e98097488d34e68623d0");
        let peer = HostedPeer::new(
            "-AR1234-111111111111",
            "127.0.0.1",
            &8080,
            Event::Started,
            999,
        );
        torrent.add_peer(peer);
        assert_eq!(torrent.get_leechers(), 1);
    }
}
