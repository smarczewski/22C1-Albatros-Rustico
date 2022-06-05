use crate::bencode_type::BencodeType;
use crate::encoding_decoding::bencode_parser::BencodeParser;
use crate::encoding_decoding::encoder::Encoder;
use crate::errors::ClientError;
use sha1::{Digest, Sha1};

pub struct TorrentInformation {
    name: String,
    announce: String,
    info_hash: Vec<u8>,
    piece_length: u32,
    length: u32,
    hashes_list: Vec<u8>,
}

impl TorrentInformation {
    pub fn new(torrent_path: &str) -> Result<TorrentInformation, ClientError> {
        let benc_torrent = BencodeParser
            .parse_file(torrent_path)
            .map_err(ClientError::NoSuchTorrentFile)?;
        let announce = read_announce(&benc_torrent)?;

        let info_value = benc_torrent
            .get_value_from_dict("info")
            .map_err(ClientError::TorrentInInvalidFormat)?;
        let name = read_name(&info_value)?;
        let length = read_length(&info_value)? as u32;
        let piece_length = read_piece_length(&info_value)? as u32;
        let hashes_list = read_hashes_list(&info_value)?;

        let benc_info_value = Encoder.bencode(&info_value);
        let mut hasher = Sha1::new();
        hasher.update(benc_info_value);
        let info_hash = hasher.finalize();

        Ok(TorrentInformation {
            name,
            announce,
            info_hash: info_hash.to_vec(),
            piece_length,
            length,
            hashes_list,
        })
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

    pub fn get_hash(&self, piece_idx: u32) -> Vec<u8> {
        let hashes_list_aux = self.hashes_list.clone();
        let (_, vec) = hashes_list_aux.split_at((20 * piece_idx) as usize);
        let (hash, _) = vec.split_at(20_usize);
        hash.to_vec()
    }
}

fn read_announce(torrent: &BencodeType) -> Result<String, ClientError> {
    let url_aux = torrent
        .get_value_from_dict("announce")
        .map_err(ClientError::TorrentInInvalidFormat)?
        .get_string()
        .map_err(ClientError::TorrentInInvalidFormat)?;

    String::from_utf8(url_aux).map_err(ClientError::StrConvertionError)
}

fn read_name(info_dict: &BencodeType) -> Result<String, ClientError> {
    let name = info_dict
        .get_value_from_dict("name")
        .map_err(ClientError::TorrentInInvalidFormat)?
        .get_string()
        .map_err(ClientError::TorrentInInvalidFormat)?;

    String::from_utf8(name).map_err(ClientError::StrConvertionError)
}

fn read_length(info_dict: &BencodeType) -> Result<i64, ClientError> {
    info_dict
        .get_value_from_dict("length")
        .map_err(ClientError::TorrentInInvalidFormat)?
        .get_integer()
        .map_err(ClientError::TorrentInInvalidFormat)
}

fn read_piece_length(info_dict: &BencodeType) -> Result<i64, ClientError> {
    info_dict
        .get_value_from_dict("piece length")
        .map_err(ClientError::TorrentInInvalidFormat)?
        .get_integer()
        .map_err(ClientError::TorrentInInvalidFormat)
}

fn read_hashes_list(info_dict: &BencodeType) -> Result<Vec<u8>, ClientError> {
    info_dict
        .get_value_from_dict("pieces")
        .map_err(ClientError::TorrentInInvalidFormat)?
        .get_string()
        .map_err(ClientError::TorrentInInvalidFormat)
}