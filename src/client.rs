use crate::p2p_messages::interested::InterestedMessage;
use crate::p2p_messages::message_trait::Message;
use crate::p2p_messages::not_interested::NotInterestedMessage;

use std::collections::HashMap;
use std::io::Error;
use std::io::ErrorKind;
use std::net::TcpStream;
use std::thread;

pub struct Client {
    _logs_dir_path: String,
    _download_dir_path: String,
}

impl Client {
    pub fn new(settings: &HashMap<String, String>) -> Result<Client, Error> {
        let log = settings.get(&"logs_dir_path".to_string());
        let download = settings.get(&"download_dir_path".to_string());

        match (log, download) {
            (Some(l), Some(d)) => Ok(Client {
                _logs_dir_path: l.clone(),
                _download_dir_path: d.clone(),
            }),
            _ => Err(Error::new(ErrorKind::InvalidInput, "Invalid settings")),
        }
    }

    pub fn run_client(&self) -> Result<(), Error> {
        let address = "127.0.0.1:8080";
        //then, we have to expand the functionality so that it can connect to multiple servers (peers).
        // do something -> send and receive messages.
        let a_connection = thread::spawn(move || {
            let stream = TcpStream::connect(address);
            if let Ok(mut s) = stream {
                let msg = InterestedMessage::new().unwrap(); //envio de prueba
                msg.send_msg(&mut s).unwrap();
            }
        });

        let another_connection = thread::spawn(move || {
            let stream = TcpStream::connect(address);
            if let Ok(mut s) = stream {
                let msg = NotInterestedMessage::new().unwrap(); //envio de prueba
                msg.send_msg(&mut s).unwrap();
            }
        });

        a_connection.join().unwrap();
        another_connection.join().unwrap();
        Ok(())
    }
}
