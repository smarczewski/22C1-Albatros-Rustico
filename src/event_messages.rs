use crate::bittorrent_client::peer::Peer;
use crate::piece::Piece;

pub enum NewEvent {
    NewConnection(Peer),
    ConnectionDropped,
    NewDownloadedPiece(Piece),
    CannotDownloadPiece(Piece),
}
