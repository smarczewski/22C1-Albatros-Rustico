use crate::bittorrent_client::client::Client;
use crate::errors::MessageError;
use crate::p2p_messages::message_trait::Message;

use std::io::{Read, Write};

#[derive(Debug, PartialEq)]
pub struct Handshake {
    pstrlen: u8,
    pstr: Vec<u8>,
    reserved: Vec<u8>,
    info_hash: Vec<u8>,
    peer_id: Vec<u8>,
}

impl Handshake {
    /// Create and returns a Handshake.
    pub fn new(sender: &Client, pstr: &str) -> Handshake {
        let torrent_info = sender.get_torrent_info();
        Handshake {
            pstrlen: pstr.len() as u8,
            pstr: pstr.as_bytes().to_vec(),
            reserved: vec![0; 8],
            info_hash: torrent_info.get_info_hash(),
            peer_id: sender.get_peer_id(),
        }
    }

    /// Reads a Handshake from a stream and returns it.
    pub fn read_msg(stream: &mut dyn Read) -> Result<Handshake, MessageError> {
        let mut pstrlen = [0u8; 1];
        stream
            .read_exact(&mut pstrlen)
            .map_err(MessageError::ReadingError)?;
        let mut pstr = vec![0; u8::from_be_bytes(pstrlen) as usize];
        stream
            .read_exact(&mut pstr)
            .map_err(MessageError::ReadingError)?;
        let mut reserved = [0u8; 8];
        stream
            .read_exact(&mut reserved)
            .map_err(MessageError::ReadingError)?;
        let mut info_hash = [0u8; 20];
        stream
            .read_exact(&mut info_hash)
            .map_err(MessageError::ReadingError)?;
        let mut peer_id = [0u8; 20];
        stream
            .read_exact(&mut peer_id)
            .map_err(MessageError::ReadingError)?;

        Ok(Handshake {
            pstrlen: pstrlen[0],
            pstr: pstr.to_vec(),
            reserved: reserved.to_vec(),
            info_hash: info_hash.to_vec(),
            peer_id: peer_id.to_vec(),
        })
    }

    pub fn is_valid(&self, info_hash: Vec<u8>, peer_id: Vec<u8>) -> bool {
        self.info_hash == info_hash && self.peer_id == peer_id
    }
}

impl Message for Handshake {
    fn print_msg(&self) {
        println!("Type: Handshake!\n",);
        println!(
            "pstr: {} {:?} {} {}\n",
            std::string::String::from_utf8_lossy(&self.pstr),
            self.reserved,
            std::string::String::from_utf8_lossy(&self.info_hash),
            std::string::String::from_utf8_lossy(&self.peer_id)
        );
        println!("================================================================\n");
    }

    /// Writes the bytes of a Handshake in the received stream.
    fn send_msg(&self, stream: &mut dyn Write) -> Result<(), MessageError> {
        stream
            .write_all(&[self.pstrlen])
            .map_err(MessageError::SendingError)?;
        stream
            .write_all(&self.pstr)
            .map_err(MessageError::SendingError)?;
        stream
            .write_all(&self.reserved)
            .map_err(MessageError::SendingError)?;
        stream
            .write_all(&self.info_hash)
            .map_err(MessageError::SendingError)?;
        stream
            .write_all(&self.peer_id)
            .map_err(MessageError::SendingError)?;

        stream.flush().unwrap();

        Ok(())
    }
}
