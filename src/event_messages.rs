use crate::bittorrent_client::{peer::Peer, piece_queue::Piece};

pub enum NewEvent {
    NewConnection(Peer),
    ConnectionDropped,
    NewDownloadedPiece(Piece),
    CannotDownloadPiece(Piece),
}

// pub struct NewConnection;

// pub struct NewPiece {
//     piece_index: u32,
// }

// impl NewPiece {
//     /// Create and returns a New Piece Message.
//     pub fn new(piece_index: u32) -> NewPiece {
//         NewPiece { piece_index }
//     }

//     pub fn get_piece_index(&self) -> u32 {
//         self.piece_index
//     }
// }

// pub struct ConnectionDropped;
