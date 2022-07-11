use crate::bitfield::PieceBitfield;
use crate::settings::Settings;
use crate::torrent_info::TorrentInfo;

use std::io::Error;
use std::net::TcpListener;
use std::sync::mpsc::Sender;
use std::sync::{Arc, RwLock};
use std::thread;

use super::peer_connection::PeerConnection;

#[derive(Debug, Clone)]
pub struct Server {
    settings: Arc<Settings>,
    torrents: Vec<(TorrentInfo, Arc<RwLock<PieceBitfield>>)>,
}

/// # struct PeerConnection
/// Represents the BitTorrent client.
/// Fields:
///     - settings
///     - tx_logger
///     - torrents -> All torrents and Bitfield
impl Server {
    /// Creates and runs a server.
    pub fn init(
        settings: Arc<Settings>,
        tx_logger: Sender<String>,
        torrents: Vec<(TorrentInfo, Arc<RwLock<PieceBitfield>>)>,
    ) {
        let server = Server::new(settings, torrents);
        let _ = server.run_server(tx_logger);
    }

    /// Receives the settings and the information of all torrents.
    /// Returns a server which is correctly initialized
    pub fn new(
        settings: Arc<Settings>,
        torrents: Vec<(TorrentInfo, Arc<RwLock<PieceBitfield>>)>,
    ) -> Server {
        Server { settings, torrents }
    }

    /// The server runs. It implies:
    ///     - Listening for new connections
    ///     - Handling the connections
    pub fn run_server(&self, tx_logger: Sender<String>) -> Result<(), Error> {
        let listener = TcpListener::bind("127.0.0.1:".to_string() + &self.settings.get_tcp_port())?;
        for new_stream in listener.incoming().flatten() {
            let dl_dir = self.settings.get_downloads_dir();
            let sh_torrents = self.torrents.clone();
            let sh_tx_logger = Sender::clone(&tx_logger);
            thread::spawn(move || {
                if let Ok(mut peer_connection) =
                    PeerConnection::new(new_stream, sh_torrents, dl_dir, sh_tx_logger)
                {
                    peer_connection.handle_connection();
                }
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::p2p_messages::handshake::Handshake;
    use crate::p2p_messages::interested::InterestedMsg;
    use crate::p2p_messages::message_builder::{MessageBuilder, P2PMessage};
    use crate::p2p_messages::message_trait::Message;
    use crate::p2p_messages::request::RequestMsg;
    use crate::piece::Piece;
    use sha1::{Digest, Sha1};
    use std::net::TcpStream;
    use std::{sync::mpsc::channel, vec};

    use crate::{errors::HandleError, settings::Settings, torrent_finder::TorrentFinder};

    fn request_a_piece(stream: &mut TcpStream, piece: &mut Piece) {
        while piece.get_rq() < piece.get_tl() {
            let begin = piece.get_rq();
            let block_length = piece.next_block_length();

            if let Ok(request_msg) = RequestMsg::new(piece.get_idx(), begin, block_length) {
                if request_msg.send_msg(stream).is_ok() {
                    piece.add_to_rq(block_length);
                }
            }
        }
    }

    // Testing:
    //  - Server creation
    //  - Listening connections
    //  - Handle connection
    //      - Handshakes + bitfield msg
    //      - Response requests with a valid piece
    #[test]
    fn integration_test_send_piece() {
        let settings = Arc::new(
            Settings::new("files_for_testing/settings_files_testing/settings.txt").handle_error(),
        );
        let torrent_path =
            "files_for_testing/torrents_testing/ubuntu-20.04.4-desktop-amd64.iso.torrent";
        let torrent = TorrentInfo::new(torrent_path).expect("It shouldn't fail");

        let mut requested_piece = Piece::new(0, torrent.get_piece_length(), torrent.get_hash(0));
        // Running the server
        let _thread = thread::spawn(move || {
            if let Ok(vec) = TorrentFinder::find(torrent_path, "files_for_testing/downloaded_files")
            {
                let (tx, _rx) = channel();
                Server::init(settings.clone(), tx, vec);
            }
        });

        thread::sleep(std::time::Duration::new(2, 0));
        // Emulating a client
        if let Ok(mut stream) = TcpStream::connect("127.0.0.1:8080") {
            let info_hash = torrent.get_info_hash();
            if Handshake::new_from_param("BitTorrent protocol", info_hash.clone(), vec![0u8; 20])
                .send_msg(&mut stream)
                .is_ok()
            {
                if let Ok(handshake_res) = Handshake::read_msg(&mut stream) {
                    if !handshake_res.is_valid(info_hash.clone()) {
                        assert!(false);
                        return;
                    }
                }
                // Receiving Bitfield
                let mut bf = vec![];
                if let Ok(p2p_msg) = MessageBuilder::build(&mut stream) {
                    match p2p_msg {
                        P2PMessage::Bitfield(msg) => bf = msg.get_pieces(),
                        _ => {
                            assert!(false);
                            return;
                        }
                    }
                }

                let mut expected_bf = PieceBitfield::new(torrent.get_n_pieces());
                expected_bf.add_a_piece(0);
                expected_bf.add_a_piece(10);
                assert_eq!(expected_bf.get_vec(), bf);

                // Sending Interested Message
                let _ = InterestedMsg::new().send_msg(&mut stream);

                // Receiving unchoke
                if let Ok(p2p_msg) = MessageBuilder::build(&mut stream) {
                    match p2p_msg {
                        P2PMessage::Unchoke(_msg) => assert!(true),
                        _ => {
                            assert!(false);
                            return;
                        }
                    }
                }
                // Request the piece
                request_a_piece(&mut stream, &mut requested_piece);

                // Receiving the piece
                while requested_piece.get_dl() < requested_piece.get_tl() {
                    if let Ok(p2p_msg) = MessageBuilder::build(&mut stream) {
                        match p2p_msg {
                            P2PMessage::Piece(msg) => {
                                requested_piece.add_block(msg.get_block());
                                requested_piece.add_to_dl(msg.get_block().len() as u32);
                            }
                            _ => {
                                assert!(false);
                                return;
                            }
                        }
                    }
                }

                // Final check
                assert_eq!(requested_piece.get_dl(), torrent.get_piece_length());
                let mut hasher = Sha1::new();
                hasher.update(requested_piece.get_data());
                let piece_hash = hasher.finalize();
                assert_eq!(requested_piece.get_hash(), piece_hash.to_vec());
            }
        } else {
            assert!(false);
        }
    }
}
