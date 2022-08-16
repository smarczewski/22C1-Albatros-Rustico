use crate::errors::MessageError;
use crate::p2p_messages::message_trait::Message;
use std::io::Write;

#[derive(Debug, PartialEq)]
pub struct UnchokeMsg {
    _length: u32,
    id: u8,
}

impl UnchokeMsg {
    /// Create and returns a Unchoke Message.
    pub fn new() -> UnchokeMsg {
        UnchokeMsg { _length: 1, id: 1 }
    }

    /// Reads a Unchoke Message from a stream and returns the message.
    pub fn read_msg(length: u32) -> Result<UnchokeMsg, MessageError> {
        if length != 1 {
            return Err(MessageError::CreationError);
        }

        Ok(UnchokeMsg::new())
    }
}

impl Message for UnchokeMsg {
    /// Writes the bytes of a Unchoke Message in the received stream.
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

impl Default for UnchokeMsg {
    fn default() -> Self {
        Self::new()
    }
}
