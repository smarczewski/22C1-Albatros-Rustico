use crate::p2p_messages::errors::MessageError;
use crate::p2p_messages::message_trait::Message;
use std::io::Write;

#[derive(Debug, PartialEq)]
pub struct NotInterestedMessage {
    _length: u32,
    id: u8,
}
impl NotInterestedMessage {
    pub fn new() -> Result<NotInterestedMessage, MessageError> {
        Ok(NotInterestedMessage { _length: 1, id: 3 })
    }

    pub fn read_msg(length: u32) -> Result<NotInterestedMessage, MessageError> {
        if length != 1 {
            return Err(MessageError::CreationError);
        }

        NotInterestedMessage::new()
    }
}

impl Message for NotInterestedMessage {
    fn print_msg(&self) {
        println!("Type: NotInterested!\n ID: {}\n", self.id);
        println!("================================================================\n");
    }

    fn send_msg(&self, stream: &mut dyn Write) -> Result<(), MessageError> {
        stream
            .write_all(&self._length.to_be_bytes())
            .map_err(MessageError::SendingError)?;
        stream
            .write_all(&self.id.to_be_bytes())
            .map_err(MessageError::SendingError)?;

        Ok(())
    }
}
