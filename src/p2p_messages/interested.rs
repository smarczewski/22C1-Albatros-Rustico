use crate::errors::MessageError;
use crate::p2p_messages::message_trait::Message;
use std::io::Write;

#[derive(Debug, PartialEq)]
pub struct InterestedMsg {
    _length: u32,
    id: u8,
}

impl InterestedMsg {
    /// Create and returns a Interested Message.
    pub fn new() -> InterestedMsg {
        InterestedMsg { _length: 1, id: 2 }
    }

    /// Reads a Interested Message from a stream and returns the message.
    pub fn read_msg(length: u32) -> Result<InterestedMsg, MessageError> {
        if length != 1 {
            return Err(MessageError::CreationError);
        }

        Ok(InterestedMsg::new())
    }
}

impl Message for InterestedMsg {
    /// Writes the bytes of a Interested Message in the received stream.
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

impl Default for InterestedMsg {
    fn default() -> Self {
        Self::new()
    }
}
