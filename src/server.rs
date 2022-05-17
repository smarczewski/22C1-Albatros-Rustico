use crate::p2p_messages::message_builder::MessageBuilder;
use std::collections::HashMap;
use std::io::Error;
use std::io::ErrorKind;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::Arc;
use std::thread;

pub struct Server {
    tcp_port: String,
}

impl Server {
    pub fn new(settings: &HashMap<String, String>) -> Result<Server, Error> {
        let tcp_port = settings.get(&"tcp_port".to_string());
        match tcp_port {
            Some(p) => Ok(Server {
                tcp_port: p.clone(),
            }),
            _ => Err(Error::new(ErrorKind::InvalidInput, "Invalid settings")),
        }
    }

    pub fn run_server(self) -> Result<(), Error> {
        let listener = TcpListener::bind(&self.tcp_port)?;
        let mut connections = Vec::new();
        let shared = Arc::new(self);

        for stream in listener.incoming() {
            if let Ok(mut s) = stream {
                let sv_shared = shared.clone();
                let current_connection = thread::spawn(move || {
                    sv_shared.handle_connection(&mut s);
                });
                connections.push(current_connection);
            } else {
                println!("Cannot accept incoming connection\n");
            }
        }

        for connection in connections {
            connection.join().unwrap();
        }

        Ok(())
    }

    fn handle_connection(&self, stream: &mut TcpStream) {
        loop {
            if let Ok(msg) = MessageBuilder::build(stream) {
                msg.print_msg();
                // msg.response()
            }
        }
    }
}
