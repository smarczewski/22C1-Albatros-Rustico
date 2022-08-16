use crate::errors::MessageError;
use crate::p2p_messages::message_trait::Message;
use std::io::{Read, Write};

#[derive(Debug, PartialEq)]
pub struct HaveMsg {
    _length: u32,
    id: u8,
    piece_index: u32,
}

impl HaveMsg {
    /// Create and returns a Have Message.
    pub fn new(piece_index: u32) -> HaveMsg {
        HaveMsg {
            _length: 5,
            id: 4,
            piece_index,
        }
    }

    /// Reads a Have Message from a stream and returns the message.
    pub fn read_msg(length: u32, stream: &mut dyn Read) -> Result<HaveMsg, MessageError> {
        if length != 5 {
            return Err(MessageError::CreationError);
        }

        let mut buf = [0u8; 4];
        stream
            .read_exact(&mut buf)
            .map_err(MessageError::ReadingError)?;

        Ok(HaveMsg::new(u32::from_be_bytes(buf)))
    }

    /// Returns the index of the piece
    pub fn get_piece_index(&self) -> u32 {
        self.piece_index
    }
}

impl Message for HaveMsg {
    /// Writes the bytes of a Have Message in the received stream.
    fn send_msg(&self, stream: &mut dyn Write) -> Result<(), MessageError> {
        stream
            .write_all(&self._length.to_be_bytes())
            .map_err(MessageError::SendingError)?;
        stream
            .write_all(&self.id.to_be_bytes())
            .map_err(MessageError::SendingError)?;
        stream
            .write_all(&self.piece_index.to_be_bytes())
            .map_err(MessageError::SendingError)?;
        let _ = stream.flush();

        Ok(())
    }
}
