use crate::bitfield::PieceBitfield;
use crate::piece::Piece;
use crate::torrent_info::TorrentInfo;

use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
};

/// # struct Piece Queue
/// Represents a queue containing the pieces that have not yet been downloaded
#[derive(Debug)]
pub struct PieceQueue(VecDeque<Piece>);

impl PieceQueue {
    /// Creates the queue using the information of the torrent and a bitfield that contains the
    /// downloaded pieces.
    pub fn new(torrent_info: &TorrentInfo, bitfield: &Arc<RwLock<PieceBitfield>>) -> PieceQueue {
        let piece_bytes = torrent_info.get_piece_length();
        let n_pieces = torrent_info.get_n_pieces();
        let mut pieces: VecDeque<Piece> = VecDeque::new();
        let bitfield_lock = match bitfield.read() {
            Ok(bf) => (&*bf).clone(),
            Err(_) => PieceBitfield::new(n_pieces),
        };
        // We create all pieces except the last one, then we push them into the queue
        for idx in 0..n_pieces - 1 {
            if bitfield_lock.has_piece(idx) {
                continue;
            }
            let piece = Piece::new(idx, piece_bytes, torrent_info.get_hash(idx));
            pieces.push_back(piece);
        }

        // We create the last piece with the correct length and  then we push it into the queue
        if !bitfield_lock.has_piece(n_pieces - 1) {
            let l_piece_bytes = match torrent_info.get_length() % piece_bytes {
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
        PieceQueue(pieces)
    }

    pub fn get_next_piece(&mut self) -> Option<Piece> {
        self.0.pop_front()
    }

    pub fn push_back(&mut self, mut piece: Piece) {
        piece.reset_info();
        self.0.push_back(piece)
    }

    pub fn length(&self) -> u32 {
        self.0.len() as u32
    }
}
