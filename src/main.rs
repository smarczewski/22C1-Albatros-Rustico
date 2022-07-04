use c122_albatros_rustico::bittorrent_client::client::Client;
use c122_albatros_rustico::channel_msg_log::logger_recv_channel::LoggerRecvChannel;
use c122_albatros_rustico::constants::MAX_CONCURRENT_TORRENTS;
use c122_albatros_rustico::errors::HandleError;
use c122_albatros_rustico::settings::Settings;
use c122_albatros_rustico::torrent_finder::TorrentFinder;
// use c122_albatros_rustico::server::Server;
use std::sync::mpsc::Sender;
use std::sync::{mpsc, Mutex};

use std::env;
use std::sync::Arc;
use std::thread;

//use c122_albatros_rustico::constants::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Invalid number of arguments");
        return;
    }
    let settings = Arc::new(Settings::new(&args[2]).handle_error());
    let torrents =
        TorrentFinder::find(&args[1].clone(), &settings.get_downloads_dir()).handle_error();
    let sh_torrents = Arc::new(Mutex::new(torrents));
    let (tx_cl_to_sv, _rx_sv) = mpsc::channel();
    // let (tx_cl_to_gui, rx_gui) = mpsc::channel();

    let (tx_logger, mut logger) = LoggerRecvChannel::new(&settings.get_log_dir()).handle_error();

    // Client execution
    let mut cl_threads = vec![];
    for _ in 0..MAX_CONCURRENT_TORRENTS {
        let tx_logger_cl = Sender::clone(&tx_logger);
        let sh_tx_cl_to_sv = Sender::clone(&tx_cl_to_sv);
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
                        Sender::clone(&sh_tx_cl_to_sv),
                        Sender::clone(&tx_logger_cl),
                    )
                }
                _ => break,
            }
        });
        cl_threads.push(client_thread);
    }

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
    // server_thread.join().unwrap();
}

/*
   // for i in 0..3{
   //     thread = thread::spawn({
   //          loop{
   //             match torrents.lock.pop()
   //                 Some    cliente = crear_cliente
   //                         run cliente

   //                 Err break;

   //         }
   //     })
   // }
*/

// let settings_sv = settings.clone();
// let server_thread = thread::spawn(move || {
//     let server = Server::new(&settings_sv).unwrap();
//     server.run_server().unwrap();
// });

// let log_path = settings.get(&"logs_dir_path".to_string()).unwrap();
// let _logger = Logger::logger_create("DEBUG", &settings.get_log_dir()).unwrap();
// let (tx, _rx): (Sender<String>, Receiver<String>) = mpsc::channel();
// let mut _logger_rcv_cnl = LoggerRecvChannel::new(_rx, _logger);
