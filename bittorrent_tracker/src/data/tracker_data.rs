use crate::{
    data::{hosted_peer::HostedPeer, hosted_torrent::HostedTorrent},
    encoding::encoder::Encoder,
    errors::TrackerError,
};
use serde::{Deserialize, Serialize};

/// # struct TrackerData
/// Represents the data that the tracker contains.
#[derive(Serialize, Deserialize, Debug)]
pub struct TrackerData {
    torrents: Vec<HostedTorrent>,
}

impl TrackerData {
    pub fn new() -> TrackerData {
        let torrents: Vec<HostedTorrent> = vec![];

        TrackerData { torrents }
    }

    pub fn update(&mut self) {
        for torrent in &mut self.torrents {
            torrent.update();
        }
    }

    /// Adds either a new torrent or a new peer to an already hosted torrent
    pub fn add_torrent(&mut self, searched_torr: &str, peer: HostedPeer) {
        // If the torrent is already on our torrent list
        for torr in &mut self.torrents {
            if torr.get_infohash() == searched_torr {
                torr.add_peer(peer);
                return;
            }
        }
        // The torrent is not on our last, therefore the peer isnt either
        let mut torrent = HostedTorrent::new(searched_torr);
        torrent.add_peer(peer);
        self.torrents.push(torrent);
    }

    /// Removes this peer from the list of peers belonging to this torrent
    pub fn remove_peer(&mut self, searched_torr: &str, peer: HostedPeer) {
        for torr in &mut self.torrents {
            if torr.get_infohash() == searched_torr {
                torr.remove_peer(peer);
                return;
            }
        }
    }

    /// Returns a bencoded dictionary that represents the tracker data
    pub fn bencode_data(&self, info_hash: String) -> Result<Vec<u8>, TrackerError> {
        for torrent in &self.torrents {
            if torrent.get_infohash() == info_hash {
                let torr_benc_type = torrent.to_bencode_type();
                let bencoded_data = Encoder.bencode(&torr_benc_type);
                return Ok(bencoded_data);
            }
        }

        Err(TrackerError::NoSuchTorrent)
    }
}

impl Default for TrackerData {
    fn default() -> Self {
        Self::new()
    }
}
