use crate::p2p_messages::errors::MessageError;
use crate::p2p_messages::message_trait::Message;
use std::io::{Read, Write};

#[derive(Debug, PartialEq)]
pub struct HaveMessage {
    _length: u32,
    id: u8,
    piece_index: u32,
}

impl HaveMessage {
    pub fn new(piece_index: u32) -> Result<HaveMessage, MessageError> {
        Ok(HaveMessage {
            _length: 5,
            id: 4,
            piece_index,
        })
    }

    pub fn read_msg(length: u32, stream: &mut dyn Read) -> Result<HaveMessage, MessageError> {
        if length != 5 {
            return Err(MessageError::CreationError);
        }

        let mut buf = [0u8; 4];
        stream
            .read_exact(&mut buf)
            .map_err(MessageError::ReadingError)?;

        HaveMessage::new(u32::from_be_bytes(buf))
    }
}

impl Message for HaveMessage {
    fn print_msg(&self) {
        println!("Type: Have!\n ID: {}\n", self.id);
        println!("Piece index: {}\n", self.piece_index);
        println!("================================================================\n");
    }

    fn send_msg(&self, stream: &mut dyn Write) -> Result<(), MessageError> {
        stream
            .write_all(&self._length.to_be_bytes())
            .map_err(MessageError::SendingError)?;
        stream
            .write_all(&self.id.to_be_bytes())
            .map_err(MessageError::SendingError)?;
        stream
            .write_all(&self.piece_index.to_be_bytes())
            .map_err(MessageError::SendingError)?;
        stream.flush().unwrap();

        Ok(())
    }
}
