use c122_albatros_rustico::bittorrent_client::client::Client;
use c122_albatros_rustico::channel_msg_log::logger_recv_channel::LoggerRecvChannel;
use c122_albatros_rustico::encoding_decoding::settings_parser::SettingsParser;
use c122_albatros_rustico::logger::Logger;
// use c122_albatros_rustico::server::Server;
//use std::sync::mpsc;
//use std::sync::mpsc::{Receiver, Sender};

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

    let torrent_path = args[1].clone();
    let settings_path = &args[2];
    let parsed_settings = SettingsParser.parse_file(settings_path);
    if let Err(error) = parsed_settings {
        error.print_error();
        return;
    }

    let settings = Arc::new(parsed_settings.expect("This shouldn't be possible"));

    // let settings_sv = settings.clone();
    // let server_thread = thread::spawn(move || {
    //     let server = Server::new(&settings_sv).unwrap();
    //     server.run_server().unwrap();
    // });

    let log_path = settings.get(&"logs_dir_path".to_string()).unwrap();
    let _logger = Logger::logger_create(log_path).unwrap();

    let (tx, mut _logger_rcv_cnl) = LoggerRecvChannel::new(log_path).unwrap();
    let settings_cl = settings;
    let client_thread = thread::spawn(move || {
        let client = Client::new(&settings_cl, torrent_path);
        match client {
            Ok(mut new_client) => {
                if let Err(error) = new_client.run_client(tx) {
                    error.print_error();
                }
            }
            Err(error) => error.print_error(),
        }
    });

    let logger_thread = thread::spawn(move || {
        while _logger_rcv_cnl.continue_receiving() {
            if _logger_rcv_cnl.receive().is_err() {
                break;
            }
        }
    });

    client_thread.join().unwrap();
    logger_thread.join().unwrap();
    // server_thread.join().unwrap();
}
