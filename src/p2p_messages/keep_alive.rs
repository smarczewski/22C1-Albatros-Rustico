use crate::errors::MessageError;
use crate::p2p_messages::message_trait::Message;
use std::io::Write;

#[derive(Debug, PartialEq)]
pub struct KeepAliveMessage {
    _length: u32,
}

impl KeepAliveMessage {
    /// Create and returns a KeepAlive Message.
    pub fn new() -> KeepAliveMessage {
        KeepAliveMessage { _length: 0 }
    }
}

impl Message for KeepAliveMessage {
    fn print_msg(&self) {
        println!("Type: KeepAlive!\n");
        println!("================================================================\n");
    }

    /// Writes the bytes of a KeepAlive Message in the received stream.
    fn send_msg(&self, stream: &mut dyn Write) -> Result<(), MessageError> {
        stream
            .write_all(&self._length.to_be_bytes())
            .map_err(MessageError::SendingError)?;
        stream.flush().unwrap();

        Ok(())
    }
}

impl Default for KeepAliveMessage {
    fn default() -> Self {
        Self::new()
    }
}
