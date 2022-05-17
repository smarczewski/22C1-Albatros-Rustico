use c122_albatros_rustico::client::Client;
use c122_albatros_rustico::parsers::settings::SettingsParser;
use c122_albatros_rustico::server::Server;

use std::env;
use std::sync::Arc;
use std::thread;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Invalid number of arguments");
        return;
    }

    let settings_path = &args[1];
    let settings = Arc::new(SettingsParser(settings_path).parse_file().unwrap());

    let settings_sv = settings.clone();
    let server_thread = thread::spawn(move || {
        let server = Server::new(&settings_sv).unwrap();
        server.run_server().unwrap();
    });

    let settings_cl = settings;
    let client_thread = thread::spawn(move || {
        let client = Client::new(&settings_cl).unwrap();
        for _i in 0..=4 {
            client.run_client().unwrap();
        }
    });

    client_thread.join().unwrap();
    server_thread.join().unwrap();
}
