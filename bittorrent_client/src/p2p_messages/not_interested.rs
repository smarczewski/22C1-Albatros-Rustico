use crate::errors::MessageError;
use crate::p2p_messages::message_trait::Message;
use std::io::Write;

#[derive(Debug, PartialEq)]
pub struct NotInterestedMsg {
    _length: u32,
    id: u8,
}
impl NotInterestedMsg {
    /// Create and returns a NotInterested Message.
    pub fn new() -> NotInterestedMsg {
        NotInterestedMsg { _length: 1, id: 3 }
    }

    /// Reads a NotInterested Message from a stream and returns the message.
    pub fn read_msg(length: u32) -> Result<NotInterestedMsg, MessageError> {
        if length != 1 {
            return Err(MessageError::CreationError);
        }

        Ok(NotInterestedMsg::new())
    }
}

impl Message for NotInterestedMsg {
    /// Writes the bytes of a NotInterested Message in the received stream.
    fn send_msg(&self, stream: &mut dyn Write) -> Result<(), MessageError> {
        stream
            .write_all(&self._length.to_be_bytes())
            .map_err(MessageError::SendingError)?;
        stream
            .write_all(&self.id.to_be_bytes())
            .map_err(MessageError::SendingError)?;
        let _ = stream.flush();

        Ok(())
    }
}

impl Default for NotInterestedMsg {
    fn default() -> Self {
        Self::new()
    }
}
