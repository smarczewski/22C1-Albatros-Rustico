use crate::bitfield::PieceBitfield;
use crate::errors::ArgsError;
use crate::event_messages::NewEvent;
use crate::torrent_info::TorrentInfo;

use std::fs::{self, read_dir};
use std::path::Path;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex, RwLock};

type TorrentCombo = (TorrentInfo, Arc<RwLock<PieceBitfield>>);
type ReceiverOfGtkSender = Receiver<glib::Sender<NewEvent>>;
/// # struct TorrentFinder

pub struct TorrentFinder;

impl TorrentFinder {
    /// It finds all .torrent files in a directory, parses them, and builds
    /// a bitfield (per torrent) with the downloaded pieces.
    pub fn find(
        dir_path: &str,
        dl_path: &str,
        rx_gui: Arc<Mutex<ReceiverOfGtkSender>>,
    ) -> Result<Vec<TorrentCombo>, ArgsError> {
        let all_torrents = TorrentFinder::find_in(dir_path)?;
        if let Ok(rx_gui_lock) = rx_gui.lock() {
            if let Ok(tx_gtk) = rx_gui_lock.recv() {
                let mut vec_torrents = vec![];
                for torrent in all_torrents {
                    if let Ok(new_torrent) = TorrentInfo::new(&torrent) {
                        let name = new_torrent.get_name();
                        let n_pieces = new_torrent.get_n_pieces();
                        let bitfield = TorrentFinder::build_bitfield(dl_path, &name, n_pieces);
                        let _ = tx_gtk.send(NewEvent::NewTorrent(
                            new_torrent.clone(),
                            bitfield.number_of_downloaded_pieces(),
                            "Single File".to_string(),
                        ));
                        vec_torrents.push((new_torrent, Arc::new(RwLock::new(bitfield))));
                    }
                }
                return Ok(vec_torrents);
            }
        }
        Err(ArgsError::CannotConnectToGUI)
    }

    /// In case of receiving a path of a single torrent file, returns it.
    /// In the case of receiving a directory, it loops through this and its sub directories.
    /// Then, it finds torrent files and returns a vector that contains all paths of found torrent files.
    /// If the file or directory does not exist, it returns error.
    fn find_in(dir_path: &str) -> Result<Vec<String>, ArgsError> {
        if TorrentFinder::is_single_torrent(dir_path) {
            return Ok(vec![dir_path.to_string()]);
        }

        let mut files = Vec::<String>::new();
        TorrentFinder::loop_through_dir(dir_path, &mut files)?;
        Ok(files)
    }

    /// Loops through a directory and finds the torrent files inside it.
    fn loop_through_dir(dir: &str, files: &mut Vec<String>) -> Result<(), ArgsError> {
        if let Ok(curr_dir) = read_dir(dir) {
            for file in curr_dir.flatten() {
                let file_path = file.path().to_string_lossy().to_string();
                if let Ok(new_dir) = file.metadata() {
                    if new_dir.is_dir() {
                        TorrentFinder::loop_through_dir(&file_path, files)?;
                    } else if TorrentFinder::is_single_torrent(&file_path) {
                        files.push(file_path);
                    }
                }
            }
            return Ok(());
        }
        Err(ArgsError::NoTorrentDir)
    }

    /// Checks if the path ends with '.torrent' extension
    fn is_single_torrent(path: &str) -> bool {
        if let Some(extension) = Path::new(path).extension() {
            return extension == "torrent";
        }
        false
    }

    fn build_bitfield(dl_path: &str, name: &str, n_pieces: u32) -> PieceBitfield {
        if !Path::new(dl_path).exists() && fs::create_dir_all(dl_path).is_err() {
            return PieceBitfield::new(n_pieces);
        }

        if let Ok(files) = fs::read_dir(dl_path) {
            let mut bitfield = PieceBitfield::new(n_pieces);
            for file in files.flatten() {
                let file_name = file.file_name().to_string_lossy().to_string();
                let piece_name = format!("{}_piece_", name);

                if file_name.contains(&piece_name) {
                    let (_, piece_idx) = file_name.split_at(piece_name.len());
                    if let Ok(idx) = piece_idx.parse::<u32>() {
                        bitfield.add_a_piece(idx);
                    }
                } else if file_name == name {
                    return PieceBitfield::get_completed_bitfield(n_pieces);
                }
            }
            return bitfield;
        }
        PieceBitfield::new(n_pieces)
    }
}

#[cfg(test)]
mod tests {
    use glib::{MainContext, PRIORITY_DEFAULT};

    use super::*;
    use std::sync::mpsc::channel;

    #[test]
    fn get_path_single_torrent() {
        let path = "a_single_torrent_file.torrent";
        if let Ok(file) = TorrentFinder::find_in(path) {
            assert_eq!(path, file[0]);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn no_such_torrent() {
        let path = "a_single_file.txt";
        let file = TorrentFinder::find_in(path);
        assert!(file.is_err());
    }

    #[test]
    fn get_path_multiple_torrents() {
        if let Ok(files) = TorrentFinder::find_in("./files_for_testing/torrents_testing") {
            let t1 = "./files_for_testing/torrents_testing/debian-11.3.0-amd64-netinst.iso.torrent";
            let t2 =
                "./files_for_testing/torrents_testing/ubuntu-20.04.4-desktop-amd64.iso.torrent";
            assert!(files.contains(&t1.to_string()));
            assert!(files.contains(&t2.to_string()));
        } else {
            assert!(false);
        }
    }

    #[test]
    fn no_such_dir() {
        let files = TorrentFinder::find_in("./no_directory");
        assert!(files.is_err());
    }

    #[test]
    fn get_multiple_torrent_info() {
        let (tx, rx) = channel();
        let (tx_gtk, _rx_gtk) = MainContext::channel(PRIORITY_DEFAULT);
        let _ = tx.send(tx_gtk);
        if let Ok(files) = TorrentFinder::find(
            "./files_for_testing/torrents_testing",
            "./files_for_testing/downloaded_files",
            Arc::new(Mutex::new(rx)),
        ) {
            let t1p = "files_for_testing/torrents_testing/debian-11.3.0-amd64-netinst.iso.torrent";
            let torrent1 = TorrentInfo::new(t1p);

            let t2p = "files_for_testing/torrents_testing/ubuntu-20.04.4-desktop-amd64.iso.torrent";
            let torrent2 = TorrentInfo::new(t2p);

            if let (Ok(t1), Ok(t2)) = (torrent1, torrent2) {
                let bitfield1 = PieceBitfield::new(t1.get_n_pieces());
                let mut bitfield2 = PieceBitfield::new(t2.get_n_pieces());
                bitfield2.add_a_piece(0);
                bitfield2.add_a_piece(10);

                let torrent_info_vec = vec![files[0].0.clone(), files[1].0.clone()];
                assert!(torrent_info_vec.contains(&t1));
                assert!(torrent_info_vec.contains(&t2));

                if let (Ok(bf1), Ok(bf2)) = (files[0].1.read(), files[1].1.read()) {
                    let bf_vec = vec![bf1.clone(), bf2.clone()];
                    assert!(bf_vec.contains(&bitfield1));
                    assert!(bf_vec.contains(&bitfield2));
                }
                return;
            }
        }
        assert!(false);
    }

    #[test]
    fn invalid_dl_path() {
        let (tx, rx) = channel();
        let (tx_gtk, _rx_gtk) = MainContext::channel(PRIORITY_DEFAULT);
        let _ = tx.send(tx_gtk);

        let torr_dir = "./files_for_testing/torrents_testing";
        if let Ok(files) = TorrentFinder::find(torr_dir, "./no_dir", Arc::new(Mutex::new(rx))) {
            let t1p = "files_for_testing/torrents_testing/debian-11.3.0-amd64-netinst.iso.torrent";
            let torrent1 = TorrentInfo::new(t1p);

            let t2p = "files_for_testing/torrents_testing/ubuntu-20.04.4-desktop-amd64.iso.torrent";
            let torrent2 = TorrentInfo::new(t2p);

            if let (Ok(t1), Ok(t2)) = (torrent1, torrent2) {
                let bitfield1 = PieceBitfield::new(t1.get_n_pieces());
                let bitfield2 = PieceBitfield::new(t2.get_n_pieces());

                // We remove ./no_dir directory, that was created above.
                let _ = fs::remove_dir_all("./no_dir");

                let torrent_info_vec = vec![files[0].0.clone(), files[1].0.clone()];
                assert!(torrent_info_vec.contains(&t1));
                assert!(torrent_info_vec.contains(&t2));

                if let (Ok(bf1), Ok(bf2)) = (files[0].1.read(), files[1].1.read()) {
                    let bf_vec = vec![bf1.clone(), bf2.clone()];
                    assert!(bf_vec.contains(&bitfield1));
                    assert!(bf_vec.contains(&bitfield2));
                }
                return;
            }
        }
        assert!(false);
    }
}
