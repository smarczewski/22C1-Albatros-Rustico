use crate::errors::MessageError;
use crate::p2p_messages::message_trait::Message;
use std::io::Write;

#[derive(Debug, PartialEq)]
pub struct InterestedMessage {
    _length: u32,
    id: u8,
}

impl InterestedMessage {
    /// Create and returns a Interested Message.
    pub fn new() -> InterestedMessage {
        InterestedMessage { _length: 1, id: 2 }
    }

    /// Reads a Interested Message from a stream and returns the message.
    pub fn read_msg(length: u32) -> Result<InterestedMessage, MessageError> {
        if length != 1 {
            return Err(MessageError::CreationError);
        }

        Ok(InterestedMessage::new())
    }
}

impl Message for InterestedMessage {
    fn print_msg(&self) {
        println!("Type: Interested!\n ID: {}\n", self.id);
        println!("================================================================\n");
    }

    /// Writes the bytes of a Interested Message in the received stream.
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

impl Default for InterestedMessage {
    fn default() -> Self {
        Self::new()
    }
}
