use crate::errors::MessageError;
use crate::p2p_messages::message_trait::Message;
use std::io::Write;

#[derive(Debug, PartialEq)]
pub struct UnchokeMessage {
    _length: u32,
    id: u8,
}

impl UnchokeMessage {
    pub fn new() -> UnchokeMessage {
        UnchokeMessage { _length: 1, id: 1 }
    }

    pub fn read_msg(length: u32) -> Result<UnchokeMessage, MessageError> {
        if length != 1 {
            return Err(MessageError::CreationError);
        }

        Ok(UnchokeMessage::new())
    }
}

impl Message for UnchokeMessage {
    fn print_msg(&self) {
        println!("Type: Unchoke!\n ID: {}\n", self.id);
        println!("================================================================\n");
    }

    fn send_msg(&self, stream: &mut dyn Write) -> Result<(), MessageError> {
        stream
            .write_all(&self._length.to_be_bytes())
            .map_err(MessageError::SendingError)?;
        stream
            .write_all(&self.id.to_be_bytes())
            .map_err(MessageError::SendingError)?;
        stream.flush().unwrap();

        Ok(())
    }
}

impl Default for UnchokeMessage {
    fn default() -> Self {
        Self::new()
    }
}
