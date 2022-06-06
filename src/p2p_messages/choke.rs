use crate::errors::MessageError;
use crate::p2p_messages::message_trait::Message;
use std::io::Write;

#[derive(Debug, PartialEq)]
pub struct ChokeMessage {
    _length: u32,
    id: u8,
}

impl ChokeMessage {
    /// Create and returns a Choke Message.
    pub fn new() -> ChokeMessage {
        ChokeMessage { _length: 1, id: 0 }
    }

    /// Reads a Choke Message from a stream and returns the message.
    pub fn read_msg(length: u32) -> Result<ChokeMessage, MessageError> {
        if length != 1 {
            return Err(MessageError::CreationError);
        }

        Ok(ChokeMessage::new())
    }
}

impl Message for ChokeMessage {
    fn print_msg(&self) {
        println!("Type: Choke!\n ID: {}\n", self.id);
        println!("================================================================\n");
    }

    /// Writes the bytes of a Choke Message in a received stream.
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

impl Default for ChokeMessage {
    fn default() -> Self {
        Self::new()
    }
}
