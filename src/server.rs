use crate::p2p_messages::choke::ChokeMessage;
use crate::p2p_messages::keep_alive::KeepAliveMessage;
use crate::p2p_messages::message_builder::{MessageBuilder, P2PMessage};
use crate::p2p_messages::message_trait::Message;
use crate::p2p_messages::unchoke::UnchokeMessage;
use crate::thread_mgmt::threadpool::ThreadPool;
use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::net::{TcpListener, TcpStream};

pub struct Server {
    tcp_port: String,
    pub pool: ThreadPool,
}

impl Server {
    pub fn new(settings: &HashMap<String, String>) -> Result<Server, Error> {
        let tcp_port = settings.get(&"tcp_port".to_string());
        let pool = ThreadPool::new(4);
        match tcp_port {
            Some(p) => Ok(Server {
                tcp_port: p.clone(),
                pool,
            }),
            _ => Err(Error::new(ErrorKind::InvalidInput, "Invalid settings")),
        }
    }

    pub fn run_server(self) -> Result<(), Error> {
        let listener = TcpListener::bind("127.0.0.1:".to_string() + &self.tcp_port)?;

        for stream in listener.incoming() {
            let stream = stream.unwrap();

            self.pool.execute(|| {
                handle_connection(stream);
            });
        }

        println!("Shutting down server.");
        Ok(())
    }
}

fn handle_connection(mut stream: TcpStream) {
    if let Ok(msg) = MessageBuilder::build(&mut stream) {
        // test responses
        match msg {
            P2PMessage::Interested(_msg) => {
                let response = UnchokeMessage::new();
                response.send_msg(&mut stream).unwrap();
            }
            P2PMessage::NotInterested(_msg) => {
                let response = ChokeMessage::new();
                response.send_msg(&mut stream).unwrap();
            }
            _ => {
                let response = KeepAliveMessage::new();
                response.send_msg(&mut stream).unwrap();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoding_decoding::settings_parser::SettingsParser;

    #[test]
    fn server_is_created_correctly() {
        let settings = SettingsParser
            .parse_file("files_for_testing/settings_files_testing/valid_format_v2.txt")
            .unwrap();
        let server = Server::new(&settings);
        assert!(server.is_ok());
    }

    #[test]
    fn server_doesnt_run_on_invalid_port() {
        let settings = SettingsParser
            .parse_file("files_for_testing/settings_files_testing/valid_format_invalid_port.txt")
            .unwrap();
        let server = Server::new(&settings).unwrap();
        assert!(server.run_server().is_err());
    }
}
