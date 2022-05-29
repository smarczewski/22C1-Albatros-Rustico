use crate::errors::MessageError;
use crate::p2p_messages::message_trait::Message;
use std::io::Write;

#[derive(Debug, PartialEq)]
pub struct KeepAliveMessage {
    _length: u32,
}

impl KeepAliveMessage {
    pub fn new() -> Result<KeepAliveMessage, MessageError> {
        Ok(KeepAliveMessage { _length: 0 })
    }
}

impl Message for KeepAliveMessage {
    fn print_msg(&self) {
        println!("Type: KeepAlive!\n");
        println!("================================================================\n");
    }

    fn send_msg(&self, stream: &mut dyn Write) -> Result<(), MessageError> {
        stream
            .write_all(&self._length.to_be_bytes())
            .map_err(MessageError::SendingError)?;
        stream.flush().unwrap();

        Ok(())
    }
}
