use c122_albatros_rustico::bittorrent_client::client::Client;
use c122_albatros_rustico::encoding_decoding::settings_parser::SettingsParser;
// use c122_albatros_rustico::server::Server;

//agregado para meter lo del logger
use c122_albatros_rustico::channel_msg_log::logger_recv_channel::LoggerRecvChannel;
use c122_albatros_rustico::logger::Logger;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
//fin de agregado para meter lo del logger

use std::env;
use std::sync::Arc;
use std::thread;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Invalid number of arguments");
        return;
    }

    let torrent_path = args[1].clone();
    let settings_path = &args[2];
    let settings = Arc::new(SettingsParser.parse_file(settings_path).unwrap());

    // let settings_sv = settings.clone();
    // let server_thread = thread::spawn(move || {
    //     let server = Server::new(&settings_sv).unwrap();
    //     server.run_server().unwrap();
    // });

    let settings_cl = settings;
    //BEGIN: codigo para meter el logger
    let settings_copy = settings_cl.clone();
    let log_path = settings_copy.get(&"logs_dir_path".to_string()).unwrap();
    println!("{:?}",log_path );
    let _logger = Logger::logger_create("DEBUG", log_path).unwrap();
    //println!("{:?}",_logger );
    let (tx, _rx): (Sender<String>, Receiver<String>) = mpsc::channel();
    let mut logger_rcv_cnl = LoggerRecvChannel::new(_rx, _logger);
    //FIN: codigo para meter el logger
    let client_thread = thread::spawn(move || {
        let mut client = Client::new(&settings_cl, torrent_path).unwrap();
        client.run_client(tx).unwrap();
    });

    let logger_thread = thread::spawn(move || {
        while logger_rcv_cnl.get_counter() > 0 {
            if logger_rcv_cnl.receive().is_err() {
                break;
            }
        }
    });

    client_thread.join().unwrap();
    logger_thread.join().unwrap();
    // server_thread.join().unwrap();
}
