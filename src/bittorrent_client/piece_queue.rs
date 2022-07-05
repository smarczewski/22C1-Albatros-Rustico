use crate::{bitfield::PieceBitfield, bittorrent_client::torrent_info::TorrentInfo};
use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
};

use super::piece::Piece;

#[derive(Debug)]
pub struct PieceQueue {
    pending_pieces: VecDeque<Piece>,
}

impl PieceQueue {
    pub fn new(torrent_info: &TorrentInfo, bitfield: &Arc<RwLock<PieceBitfield>>) -> PieceQueue {
        let total_bytes = torrent_info.get_length();
        let piece_bytes = torrent_info.get_piece_length();
        let n_pieces = torrent_info.get_n_pieces();
        let mut pieces: VecDeque<Piece> = VecDeque::new();
        let bitfield_lock = bitfield.read().unwrap();

        // Creamos todas las pieces excepto la ultima y las pusheamos a la queue
        for idx in 0..n_pieces - 1 {
            if bitfield_lock.has_piece(idx) {
                continue;
            }
            let piece = Piece::new(idx, piece_bytes, torrent_info.get_hash(idx));
            pieces.push_back(piece);
        }

        // Creamos la ultima piece con el tamano que corresponde y pusheamos a queue
        if !bitfield_lock.has_piece(n_pieces - 1) {
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
        }

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
