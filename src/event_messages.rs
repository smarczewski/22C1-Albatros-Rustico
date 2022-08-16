use crate::bt_client::peer::Peer;
use crate::piece::Piece;
use crate::torrent_info::TorrentInfo;

pub enum NewEvent {
    NewTorrent(TorrentInfo, u32, String),
    DownloadingTorrent(String),
    TorrentDownloadFailed(String),
    NewConnection(String, Peer),
    ConnectionDropped(String, Peer),
    NewDownloadedPiece(String, Piece, Peer),
    CannotDownloadPiece(Piece),
    NumberOfPeers(String, u32),
    OurStatus(String, Peer),
}
