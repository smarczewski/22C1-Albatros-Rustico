use crate::bittorrent_client::torrent_info::TorrentInfo;
use sha1::{Digest, Sha1};
use std::collections::VecDeque;

#[derive(Debug)]
//#[derive(Clone)]
pub struct Piece {
    idx: u32,               // nro de pieza
    tl_piece_bytes: u32,    // bytes totales de la pieza
    dl_piece_bytes: u32,    // los bytes de la pieza que descargamos
    rq_piece_bytes: u32,    // los bytes que hicimos request
    expected_hash: Vec<u8>, // el hash que esperamos
    data: Vec<u8>,          // la data de la pieza (bloques)
}

impl Piece {
    pub fn new(idx: u32, tl_piece_bytes: u32, expected_hash: Vec<u8>) -> Piece {
        Piece {
            idx,
            tl_piece_bytes,
            dl_piece_bytes: 0,
            rq_piece_bytes: 0,
            expected_hash,
            data: vec![],
        }
    }

    pub fn add_block(&mut self, mut block: Vec<u8>) {
        self.data.append(&mut block)
    }

    /// Return the size of the next block
    pub fn next_block_length(&self) -> u32 {
        let block_length = 1 << 14;
        let left = self.tl_piece_bytes - self.rq_piece_bytes;
        if left < block_length {
            return left;
        }
        block_length
    }

    /// Checks if the downloaded piece is valid. To do this, it compares the hash of downloaded piece
    /// with the hash of the original piece that is in the torrent file
    pub fn piece_is_valid(&mut self) -> bool {
        //Get hash of downloaded piece
        let mut hasher = Sha1::new();
        hasher.update(self.get_data());
        let piece_hash = hasher.finalize();

        //Compare two hashes
        self.expected_hash == piece_hash.to_vec()
    }

    pub fn get_idx(&self) -> u32 {
        self.idx
    }

    pub fn get_dl(&self) -> u32 {
        self.dl_piece_bytes
    }

    pub fn get_tl(&self) -> u32 {
        self.tl_piece_bytes
    }

    pub fn get_rq(&self) -> u32 {
        self.rq_piece_bytes
    }

    pub fn add_to_dl(&mut self, bytes: u32) {
        self.dl_piece_bytes += bytes;
    }

    pub fn add_to_rq(&mut self, bytes: u32) {
        self.rq_piece_bytes += bytes;
    }

    pub fn get_data(&self) -> Vec<u8> {
        self.data.clone()
    }

    pub fn get_hash(&mut self) -> Vec<u8> {
        self.expected_hash.clone()
    }

    pub fn empty_data(&mut self) {
        self.data = vec![];
    }

    pub fn reset_info(&mut self) {
        self.dl_piece_bytes = 0;
        self.rq_piece_bytes = 0;
        self.data = vec![];
    }
}

#[derive(Debug)]
pub struct PieceQueue {
    pending_pieces: VecDeque<Piece>,
}

impl PieceQueue {
    pub fn new(torrent_info: &TorrentInfo) -> PieceQueue {
        let total_bytes = torrent_info.get_length();
        let piece_bytes = torrent_info.get_piece_length();
        let n_pieces = torrent_info.get_n_pieces();
        let mut pieces: VecDeque<Piece> = VecDeque::new();
        // Creamos todas las pieces excepto la ultima y las pusheamos a la queue
        for idx in 0..n_pieces - 1 {
            let piece = Piece::new(idx, piece_bytes, torrent_info.get_hash(idx));
            pieces.push_back(piece);
        }
        // Creamos la ultima piece con el tamano que corresponde y pusheamos a queue

        let l_piece_bytes = match total_bytes % piece_bytes {
            0 => piece_bytes,
            piece_size => piece_size,
        };

        let l_piece = Piece::new(
            n_pieces - 1,
            l_piece_bytes,
            torrent_info.get_hash(n_pieces - 1),
        );
        pieces.push_back(l_piece);

        PieceQueue {
            pending_pieces: pieces,
        }
    }

    //unused
    pub fn get_next_piece(&mut self) -> Option<Piece> {
        self.pending_pieces.pop_front()
    }

    pub fn push_back(&mut self, mut piece: Piece) {
        piece.reset_info();
        self.pending_pieces.push_back(piece)
    }

    pub fn is_empty(&self) -> bool {
        self.pending_pieces.is_empty()
    }

    pub fn length(&self) -> usize {
        self.pending_pieces.len()
    }
}
