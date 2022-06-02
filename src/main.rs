use c122_albatros_rustico::bittorrent_client::client::Client;
use c122_albatros_rustico::encoding_decoding::settings_parser::SettingsParser;
use c122_albatros_rustico::server::Server;

use std::env;
use std::sync::Arc;
use std::thread;



fn main(){
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Invalid number of arguments");
        return;
    }

    let torrent_path = args[1].clone();
    let settings_path = &args[2];
    let settings = Arc::new(SettingsParser.parse_file(settings_path).unwrap());

    let settings_sv = settings.clone();
    let server_thread = thread::spawn(move || {
        let server = Server::new(&settings_sv).unwrap();
        server.run_server().unwrap();
    });

    let settings_cl = settings;
    let client_thread = thread::spawn(move || {
        let mut client = Client::new(&settings_cl, torrent_path).unwrap();
        client.run_client().unwrap();
    });

    client_thread.join().unwrap();
    server_thread.join().unwrap();
    
}
