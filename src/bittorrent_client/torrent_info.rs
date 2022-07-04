use crate::encoding_decoding::bencode_parser::BencodeParser;
use crate::encoding_decoding::encoder::Encoder;
use sha1::{Digest, Sha1};
use std::io::{Error, ErrorKind};

/// # struct TorrentInformation
/// Contains the information of torrent file.
#[derive(Debug, Clone, PartialEq)]
pub struct TorrentInfo {
    name: String,
    announce: String,
    info_hash: Vec<u8>, // hash con el que hacemos el handshake
    piece_length: u32,
    length: u32,
    n_pieces: u32,
    hashes_list: Vec<u8>, // hashes de cada pieza para validar
}

impl TorrentInfo {
    /// Receives a path of the torrent file, then parses it.
    /// On success, returns a TorrentInformation which contains the information
    /// of the parsed torrent file.
    /// Otherwise, returns ClientError (NoSuchTorrentFile or TorrentInInvalidFormat)
    pub fn new(torrent_path: &str) -> Result<TorrentInfo, Error> {
        if let Ok(benc_torrent) = BencodeParser.parse_file(torrent_path) {
            let announce = String::from_utf8_lossy(
                &benc_torrent.get_value_from_dict("announce")?.get_string()?,
            )
            .to_string();
            let info_value = benc_torrent.get_value_from_dict("info")?;
            let name =
                String::from_utf8_lossy(&info_value.get_value_from_dict("name")?.get_string()?)
                    .to_string();
            let length = info_value.get_value_from_dict("length")?.get_integer()? as u32;
            let piece_length = info_value
                .get_value_from_dict("piece length")?
                .get_integer()? as u32;
            let n_pieces = (length as f32 / piece_length as f32).ceil() as u32;
            let hashes_list = info_value.get_value_from_dict("pieces")?.get_string()?;

            let benc_info_value = Encoder.bencode(&info_value);
            let mut hasher = Sha1::new();
            hasher.update(benc_info_value);
            let info_hash = hasher.finalize();
            return Ok(TorrentInfo {
                name,
                announce,
                info_hash: info_hash.to_vec(),
                piece_length,
                length,
                n_pieces,
                hashes_list,
            });
        }
        println!("Cannot find or parse the torrent: {}", torrent_path);
        Err(Error::new(
            ErrorKind::InvalidData,
            "Cannot find or parse torrent file",
        ))
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_announce(&self) -> String {
        self.announce.clone()
    }

    pub fn get_info_hash(&self) -> Vec<u8> {
        self.info_hash.clone()
    }

    pub fn get_length(&self) -> u32 {
        self.length
    }

    pub fn get_piece_length(&self) -> u32 {
        self.piece_length
    }

    pub fn length_of_piece_n(&self, piece_idx: u32) -> u32 {
        let last_piece_len = self.length - (self.n_pieces - 1) * self.piece_length;

        match piece_idx {
            _ if piece_idx == self.n_pieces - 1 => last_piece_len,
            _ if piece_idx < self.n_pieces - 1 => self.piece_length,
            _ => 0,
        }
    }

    pub fn get_n_pieces(&self) -> u32 {
        self.n_pieces
    }

    pub fn get_hash(&self, piece_idx: u32) -> Vec<u8> {
        let hashes_list_aux = self.hashes_list.clone();
        let (_, vec) = hashes_list_aux.split_at((20 * piece_idx) as usize);
        let (hash, _) = vec.split_at(20_usize);
        hash.to_vec()
    }

    pub fn get_hashes_list(&self) -> Vec<u8> {
        self.hashes_list.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_torrent_info_with_pieces() {
        let path = "files_for_testing/torrents_testing/ubuntu-20.04.4-desktop-amd64.iso.torrent";
        let torrent = TorrentInfo::new(path).unwrap();

        let exp_name = "ubuntu-20.04.4-desktop-amd64.iso".to_string();
        let exp_announce = "https://torrent.ubuntu.com/announce".to_string();
        let exp_infohash = vec![
            240, 156, 141, 8, 132, 89, 0, 136, 244, 0, 78, 1, 10, 146, 143, 139, 97, 120, 194, 253,
        ];
        let exp_piece_length = 262144;
        let exp_length = 3379068928;
        let n_pieces = 12891;

        assert_eq!(torrent.get_name(), exp_name);
        assert_eq!(torrent.get_announce(), exp_announce);
        assert_eq!(torrent.get_info_hash(), exp_infohash);
        assert_eq!(torrent.get_piece_length(), exp_piece_length);
        assert_eq!(torrent.get_length(), exp_length);
        assert_eq!(torrent.get_n_pieces(), n_pieces);
    }

    #[test]
    fn get_torrent_info_without_pieces() {
        let path = "files_for_testing/torrents_tracker_request_test/lubuntu-18.04-alternate-i386.iso.torrent";
        let torrent = TorrentInfo::new(path).unwrap();

        let exp_name = "lubuntu-18.04-alternate-i386.iso".to_string();
        let exp_announce = "http://torrent.ubuntu.com:6969/announce".to_string();
        let exp_infohash = vec![
            235, 149, 253, 102, 100, 184, 196, 72, 121, 43, 111, 77, 235, 35, 138, 248, 85, 20, 43,
            209,
        ];
        let exp_piece_length = 524288;
        let exp_length = 749731840;
        let n_pieces = 1430;

        assert_eq!(torrent.get_name(), exp_name);
        assert_eq!(torrent.get_announce(), exp_announce);
        assert_eq!(torrent.get_info_hash(), exp_infohash);
        assert_eq!(torrent.get_piece_length(), exp_piece_length);
        assert_eq!(torrent.get_length(), exp_length);
        assert_eq!(torrent.get_n_pieces(), n_pieces);
    }
}
