use crate::bittorrent_client::peer::Peer;
use crate::encoding_decoding::encoder::Encoder;
use crate::event_messages::NewEvent;
use crate::gui::gui_assets::{GeneralColumns, View};
use crate::piece::Piece;
use crate::torrent_info::TorrentInfo;
use gio::prelude::ApplicationExtManual;
use glib::{Continue, MainContext, PRIORITY_DEFAULT};
use gtk::prelude::*;
use gtk::Application;
use gtk::TreeIter;
use gtk::TreePath;

use std::collections::HashMap;
use std::fmt::Write;
use std::sync::mpsc::Sender;
use std::time::Instant;

use super::gui_assets::StatColumns;

pub struct UserInterface {
    view: View,
    tx: Sender<glib::Sender<NewEvent>>,
    active_connections: HashMap<String, Instant>,
}

impl UserInterface {
    pub fn run(tx: Sender<glib::Sender<NewEvent>>) {
        let app = Application::new(Some("com.taller.app"), Default::default());
        app.connect_activate(move |app| {
            let ui = UserInterface::new(app, Sender::clone(&tx));
            ui.show();
        });

        let s: [String; 0] = [];
        app.run_with_args(&s);
    }

    fn new(app: &gtk::Application, tx: Sender<glib::Sender<NewEvent>>) -> Self {
        let view = View::new(app);
        let active_connections = HashMap::new();

        UserInterface {
            view,
            tx,
            active_connections,
        }
    }

    fn show(mut self) {
        let (tx_gui, rx) = MainContext::channel(PRIORITY_DEFAULT);
        let _ = self.tx.send(glib::Sender::clone(&tx_gui));

        self.view.window.show_all();

        rx.attach(None, move |msg| {
            self.handle_msg(msg, glib::Sender::clone(&tx_gui));
            Continue(true)
        });
    }

    fn handle_msg(&mut self, msg: NewEvent, tx_gui: glib::Sender<NewEvent>) {
        match msg {
            NewEvent::NewTorrent(torrent_info, piece_count, structure) => {
                let _ = self.tx.send(glib::Sender::clone(&tx_gui));
                self.add_new_torrent(&torrent_info, piece_count, structure);
            }
            NewEvent::DownloadingTorrent(torrent_name) => {
                self.set_status(&torrent_name, "Downloading");
            }
            NewEvent::TorrentDownloadFailed(torrent_name) => {
                let _ = self.tx.send(glib::Sender::clone(&tx_gui));
                self.set_status(&torrent_name, "Paused");
            }
            NewEvent::NewConnection(torrent_name, peer) => {
                self.add_new_peer(&peer, &torrent_name);
            }
            NewEvent::ConnectionDropped(torrent_name, peer) => {
                self.delete_peer(&peer, &torrent_name);
            }
            NewEvent::NewDownloadedPiece(torrent_name, piece, peer) => {
                self.add_new_piece(&torrent_name, peer, piece);
            }
            NewEvent::NumberOfPeers(torrent_name, no_of_peers) => {
                self.set_number_of_peers(&torrent_name, no_of_peers);
            }
            NewEvent::OurStatus(status, peer) => {
                self.update_status(status, peer);
            }
            _ => (),
        }
    }

    fn add_new_torrent(&mut self, torrent_info: &TorrentInfo, piece_count: u32, structure: String) {
        let model = &self.view.notebook.general_info.list_store;

        let mut infohash = String::new();
        for byte in &torrent_info.get_info_hash() {
            let _ = write!(infohash, "%{:02x}", byte);
        }

        let size = self.convert_bytes_to_gb(torrent_info.get_length());

        // Assembling the row
        let mut status = "Paused";
        if piece_count == torrent_info.get_n_pieces() {
            status = "Finished";
        }

        let values: [(u32, &dyn ToValue); 10] = [
            (0, &torrent_info.get_name()),
            (1, &Encoder.urldecode(&infohash)),
            (2, &structure),
            (3, &size),
            (4, &torrent_info.get_n_pieces()),
            (5, &piece_count),
            (6, &0u32),
            (7, &0u32),
            (8, &status.to_string()),
            (9, &(piece_count / torrent_info.get_n_pieces() * 100)),
        ];

        model.set(&model.append(), &values);
    }

    fn set_status(&mut self, torrent_name: &String, status: &str) {
        let model = &self.view.notebook.general_info.list_store;

        if let Some(iter) = self.search_torrent(torrent_name) {
            if let Ok(current_status) = model
                .value(&iter, GeneralColumns::Status as i32)
                .get::<String>()
            {
                if current_status != "Finished" {
                    model.set_value(
                        &iter,
                        GeneralColumns::Status as i32 as u32,
                        &status.to_value(),
                    );
                }
            }
        }
    }

    fn set_number_of_peers(&mut self, torrent_name: &String, no_of_peers: u32) {
        let model = &self.view.notebook.general_info.list_store;

        if let Some(iter) = self.search_torrent(torrent_name) {
            model.set_value(
                &iter,
                GeneralColumns::Peers as i32 as u32,
                &no_of_peers.to_value(),
            );
        }
    }

    fn add_new_piece(&mut self, torrent_name: &String, peer: Peer, piece: Piece) {
        if let Some(iter) = self.search_torrent(torrent_name) {
            self.update_dl_pieces(&iter, torrent_name);
        }

        let instant = Instant::now();
        if let Some(old_instant) = self.active_connections.insert(peer.ip(), instant) {
            if let Some(duration) = instant.checked_duration_since(old_instant) {
                let speed = piece.get_tl() as f32 / 1024.0 / duration.as_secs_f32();
                if let Some(curr_peer) = self.search_peer(&peer.ip()) {
                    let model_stats = &self.view.notebook.download_stats.list_store;
                    model_stats.set_value(
                        &curr_peer,
                        StatColumns::DownloadSpeed as i32 as u32,
                        &format!("{:.2} KiB/s", speed).to_value(),
                    );
                }
            }
        }
    }

    fn search_torrent(&self, torrent: &String) -> Option<TreeIter> {
        let model = &self.view.notebook.general_info.list_store;
        let mut path = TreePath::new_first();
        let mut curr_iter = model.iter(&path);

        while let Some(iter) = curr_iter {
            if let Ok(current_torrent) = model
                .value(&iter, GeneralColumns::Name as i32)
                .get::<String>()
            {
                if &current_torrent == torrent {
                    return Some(iter);
                }
                path.next();
                curr_iter = model.iter(&path);
            }
        }
        None
    }

    fn search_peer(&self, peer_ip: &String) -> Option<TreeIter> {
        let model = &self.view.notebook.download_stats.list_store;
        let mut path = TreePath::new_first();
        let mut curr_iter = model.iter(&path);

        while let Some(iter) = curr_iter {
            if let Ok(current_peer_ip) = model
                .value(&iter, StatColumns::PeerIP as i32)
                .get::<String>()
            {
                if &current_peer_ip == peer_ip {
                    return Some(iter);
                }
                path.next();
                curr_iter = model.iter(&path);
            }
        }
        None
    }

    fn add_new_peer(&mut self, peer: &Peer, torrent: &String) {
        if let Some(iter) = self.search_torrent(torrent) {
            self.update_active_connections(&iter, true);
        }

        // Assembling the row
        let encoded_id = Encoder.urlencode(&peer.id());
        let values: [(u32, &dyn ToValue); 6] = [
            (0, &encoded_id),
            (1, &peer.ip()),
            (2, &peer.port()),
            (3, &"choked / not interested"),
            (4, &"choked / not interested"),
            (5, &"0 KiB/s"),
        ];

        let model_stats = &self.view.notebook.download_stats.list_store;
        model_stats.set(&model_stats.append(), &values);

        let instant = Instant::now();
        self.active_connections.insert(peer.ip(), instant);
    }

    fn update_active_connections(&mut self, iter: &TreeIter, increase: bool) {
        let model = &self.view.notebook.general_info.list_store;
        if let Ok(mut connections) = model
            .value(iter, GeneralColumns::ActiveConnections as i32)
            .get::<u32>()
        {
            match increase {
                true => connections += 1,
                false => connections -= 1,
            }

            model.set_value(
                iter,
                GeneralColumns::ActiveConnections as i32 as u32,
                &connections.to_value(),
            );
        }
    }

    fn delete_peer(&mut self, peer: &Peer, torrent_name: &String) {
        if let Some(iter) = self.search_torrent(torrent_name) {
            self.update_active_connections(&iter, false);
        }

        if let Some(iter) = self.search_peer(&peer.ip()) {
            let model_stats = &self.view.notebook.download_stats.list_store;
            model_stats.remove(&iter);
        }
    }

    fn update_dl_pieces(&mut self, iter: &TreeIter, torrent_name: &String) {
        let model = &self.view.notebook.general_info.list_store;

        if let Ok(mut dl_pieces) = model
            .value(iter, GeneralColumns::DownloadedPieces as i32)
            .get::<u32>()
        {
            dl_pieces += 1;
            model.set_value(
                iter,
                GeneralColumns::DownloadedPieces as i32 as u32,
                &dl_pieces.to_value(),
            );

            if let Ok(tl_pieces) = model
                .value(iter, GeneralColumns::TotalPieces as i32)
                .get::<u32>()
            {
                if dl_pieces == tl_pieces {
                    self.set_status(torrent_name, "Finished");
                }
            }

            self.update_progress_bar(iter, dl_pieces);
        }
    }

    fn update_progress_bar(&mut self, iter: &TreeIter, dl_pieces: u32) {
        let model = &self.view.notebook.general_info.list_store;
        let current_progress_res = model
            .value(iter, GeneralColumns::Progress as i32)
            .get::<u32>();

        let total_pieces_res = model
            .value(iter, GeneralColumns::TotalPieces as i32)
            .get::<u32>();

        if let (Ok(current_progress), Ok(total_pieces)) = (current_progress_res, total_pieces_res) {
            let progress = (dl_pieces * 100) / total_pieces;
            if progress <= 100 && current_progress != progress {
                model.set_value(
                    iter,
                    (GeneralColumns::Progress as i32).try_into().unwrap(),
                    &progress.to_value(),
                );
            } else if current_progress != progress {
                model.set_value(
                    iter,
                    (GeneralColumns::Progress as i32).try_into().unwrap(),
                    &100.to_value(),
                );
            }
        }
    }

    // Converts bytes to gb formatted to a String with two decimals
    fn convert_bytes_to_gb(&self, size: u32) -> String {
        let size_gb = (size as f32) / 1024.0 / 1024.0 / 1024.0;
        format!("{:.2}", size_gb)
    }

    fn update_status(&mut self, status: String, peer: Peer) {
        let model = &self.view.notebook.download_stats.list_store;

        if let Some(iter) = self.search_peer(&peer.ip()) {
            model.set_value(
                &iter,
                StatColumns::OurStatus as i32 as u32,
                &status.to_value(),
            );
        }
    }
}
