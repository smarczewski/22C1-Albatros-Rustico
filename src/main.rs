use c122_albatros_rustico::bitfield::PieceBitfield;
use c122_albatros_rustico::bittorrent_client::client::Client;
use c122_albatros_rustico::bittorrent_server::server::Server;
use c122_albatros_rustico::constants::MAX_CONCURRENT_TORRENTS;
use c122_albatros_rustico::errors::HandleError;
use c122_albatros_rustico::logging::logger_recv_channel::LoggerRecvChannel;
use c122_albatros_rustico::settings::Settings;
use c122_albatros_rustico::torrent_finder::TorrentFinder;
use c122_albatros_rustico::torrent_info::TorrentInfo;
use std::env;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;
use std::thread;
use std::thread::JoinHandle;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Invalid number of arguments");
        return;
    }
    let settings = Arc::new(Settings::new(&args[2]).handle_error());
    let torrents =
        TorrentFinder::find(&args[1].clone(), &settings.get_downloads_dir()).handle_error();
    let (tx_logger, mut logger) = LoggerRecvChannel::new(&settings.get_log_dir()).handle_error();

    server_execution(
        settings.clone(),
        Sender::clone(&tx_logger),
        torrents.clone(),
    );

    let cl_threads = client_execution(settings, Sender::clone(&tx_logger), torrents);

    let logger_thread = thread::spawn(move || {
        while logger.continue_receiving() {
            if logger.receive().is_err() {
                break;
            }
        }
    });

    for thread in cl_threads {
        thread.join().unwrap();
    }
    logger_thread.join().unwrap();
}

fn server_execution(
    settings: Arc<Settings>,
    tx_logger: Sender<String>,
    torrents: Vec<(TorrentInfo, Arc<RwLock<PieceBitfield>>)>,
) {
    let settings_sv = settings;
    let tx_logger_sv = Sender::clone(&tx_logger);
    thread::spawn(move || {
        Server::init(settings_sv, tx_logger_sv, torrents);
    });
}

fn client_execution(
    settings: Arc<Settings>,
    tx_logger: Sender<String>,
    torrents: Vec<(TorrentInfo, Arc<RwLock<PieceBitfield>>)>,
) -> Vec<JoinHandle<()>> {
    let sh_torrents = Arc::new(Mutex::new(torrents));
    let mut cl_threads = vec![];
    for _i in 0..MAX_CONCURRENT_TORRENTS {
        let sh_tx_logger = Sender::clone(&tx_logger);
        let sh_torrents_i = sh_torrents.clone();
        let settings_i = settings.clone();
        let client_thread = thread::spawn(move || loop {
            let current_torrent = sh_torrents_i.lock().unwrap().pop();
            match current_torrent {
                Some(curr_torrent) => {
                    println!("Torrent {}", curr_torrent.0.get_name());
                    Client::init(
                        settings_i.clone(),
                        curr_torrent,
                        Sender::clone(&sh_tx_logger),
                    )
                }
                _ => break,
            }
        });
        cl_threads.push(client_thread);
    }
    cl_threads
}
