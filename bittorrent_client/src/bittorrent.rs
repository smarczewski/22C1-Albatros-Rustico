use crate::bitfield::PieceBitfield;
use crate::bt_client::client::Client;
use crate::bt_server::server::Server;
use crate::constants::MAX_CONCURRENT_TORRENTS;
use crate::errors::ArgsError;
use crate::errors::HandleError;
use crate::event_messages::NewEvent;
use crate::logging::logger_recv_channel::LoggerRecvChannel;
use crate::settings::Settings;
use crate::torrent_finder::TorrentFinder;
use crate::torrent_info::TorrentInfo;

use glib;
use std::env;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;
use std::thread;
use std::thread::JoinHandle;

type TorrentCombo = (TorrentInfo, Arc<RwLock<PieceBitfield>>);

pub fn run_bittorrent(rx_gui: Receiver<glib::Sender<NewEvent>>) {
    let args = check_arguments(env::args().collect()).handle_error();

    let sh_rx_gui = Arc::new(Mutex::new(rx_gui));
    let settings = Arc::new(Settings::new(&args[2]).handle_error());
    let torrents = TorrentFinder::find(
        &args[1].clone(),
        &settings.get_downloads_dir(),
        sh_rx_gui.clone(),
    )
    .handle_error();

    let (tx_logger, mut logger) = LoggerRecvChannel::new(&settings.get_log_dir()).handle_error();

    // Handling server
    let sv_thread = handle_server(
        settings.clone(),
        Sender::clone(&tx_logger),
        torrents.clone(),
    );

    // Handling client
    let cl_threads = handle_client(
        Arc::new(Mutex::new(torrents)),
        Sender::clone(&tx_logger),
        sh_rx_gui,
        settings,
    );

    let logger_thread = thread::spawn(move || {
        while logger.continue_receiving() {
            if logger.receive().is_err() {
                break;
            }
        }
    });

    for thread in cl_threads {
        if thread.join().is_err() {
            println!("Error during client threads joining");
        }
    }
    if sv_thread.join().is_err() {
        println!("Error during server thread joining");
    }
    if logger_thread.join().is_err() {
        println!("Error during logger thread joining");
    }
}

fn check_arguments(args: Vec<String>) -> Result<Vec<String>, ArgsError> {
    if args.len() != 3 {
        return Err(ArgsError::InvalidNumberOfArguments);
    }
    Ok(args)
}

fn handle_server(
    settings: Arc<Settings>,
    tx_logger: Sender<String>,
    torrents: Vec<(TorrentInfo, Arc<RwLock<PieceBitfield>>)>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        Server::init(settings, tx_logger, torrents);
    })
}

fn handle_client(
    torrents: Arc<Mutex<Vec<TorrentCombo>>>,
    tx_logger: Sender<String>,
    rx_gui: Arc<Mutex<Receiver<glib::Sender<NewEvent>>>>,
    settings: Arc<Settings>,
) -> Vec<thread::JoinHandle<()>> {
    let mut cl_threads = vec![];

    for _i in 0..MAX_CONCURRENT_TORRENTS {
        let torrents_i = torrents.clone();
        let settings_i = settings.clone();
        let rx_gui_i = rx_gui.clone();
        let tx_logger_i = tx_logger.clone();
        let client_thread = thread::spawn(move || loop {
            let current_torrent = match torrents_i.lock() {
                Ok(mut torrents_vec) => torrents_vec.pop(),
                _ => None,
            };

            match current_torrent {
                Some(curr_torrent) => {
                    println!("Torrent {}", curr_torrent.0.get_name());
                    if let Err((torrent, pieces)) = Client::init(
                        settings_i.clone(),
                        curr_torrent,
                        Sender::clone(&tx_logger_i),
                        rx_gui_i.clone(),
                    ) {
                        if let Ok(mut torrents_vec) = torrents_i.lock() {
                            torrents_vec.push((torrent, pieces));
                        }
                    }
                }
                _ => break,
            }
        });
        cl_threads.push(client_thread);
    }

    cl_threads
}
