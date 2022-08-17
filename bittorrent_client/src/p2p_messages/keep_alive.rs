use crate::errors::MessageError;
use crate::p2p_messages::message_trait::Message;
use std::io::Write;

#[derive(Debug, PartialEq, Eq)]
pub struct KeepAliveMsg {
    _length: u32,
}

impl KeepAliveMsg {
    /// Create and returns a KeepAlive Message.
    pub fn new() -> KeepAliveMsg {
        KeepAliveMsg { _length: 0 }
    }
}

impl Message for KeepAliveMsg {
    /// Writes the bytes of a KeepAlive Message in the received stream.
    fn send_msg(&self, stream: &mut dyn Write) -> Result<(), MessageError> {
        stream
            .write_all(&self._length.to_be_bytes())
            .map_err(MessageError::SendingError)?;
        let _ = stream.flush();

        Ok(())
    }
}

impl Default for KeepAliveMsg {
    fn default() -> Self {
        Self::new()
    }
}
