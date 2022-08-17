use crate::errors::MessageError;
use crate::p2p_messages::message_trait::Message;
use std::io::Write;

#[derive(Debug, PartialEq, Eq)]
pub struct ChokeMsg {
    _length: u32,
    id: u8,
}

impl ChokeMsg {
    /// Create and returns a Choke Message.
    pub fn new() -> ChokeMsg {
        ChokeMsg { _length: 1, id: 0 }
    }

    /// Reads a Choke Message from a stream and returns the message.
    pub fn read_msg(length: u32) -> Result<ChokeMsg, MessageError> {
        if length != 1 {
            return Err(MessageError::CreationError);
        }

        Ok(ChokeMsg::new())
    }
}

impl Message for ChokeMsg {
    /// Writes the bytes of a Choke Message in a received stream.
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

impl Default for ChokeMsg {
    fn default() -> Self {
        Self::new()
    }
}
