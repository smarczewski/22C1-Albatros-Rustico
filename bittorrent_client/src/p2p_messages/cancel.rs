use crate::errors::MessageError;
use crate::p2p_messages::message_trait::Message;
use std::io::{Read, Write};

#[derive(Debug, PartialEq)]
pub struct CancelMsg {
    _length: u32,
    id: u8,
    piece_index: u32,
    begin: u32,
    block_length: u32,
}

impl CancelMsg {
    /// Create and returns a Cancel Message.
    pub fn new(piece_index: u32, begin: u32, block_length: u32) -> CancelMsg {
        CancelMsg {
            _length: 13,
            id: 8,
            piece_index,
            begin,
            block_length,
        }
    }

    /// Reads a Cancel Message from a stream and returns the message.
    pub fn read_msg(length: u32, stream: &mut dyn Read) -> Result<CancelMsg, MessageError> {
        if length != 13 {
            return Err(MessageError::CreationError);
        }

        let mut buf = [0u8; 4];
        stream
            .read_exact(&mut buf)
            .map_err(MessageError::ReadingError)?;
        let piece_index = u32::from_be_bytes(buf);
        stream
            .read_exact(&mut buf)
            .map_err(MessageError::ReadingError)?;
        let begin = u32::from_be_bytes(buf);
        stream
            .read_exact(&mut buf)
            .map_err(MessageError::ReadingError)?;
        let block_length = u32::from_be_bytes(buf);

        Ok(CancelMsg::new(piece_index, begin, block_length))
    }
}

impl Message for CancelMsg {
    /// Writes the bytes of a Cancel Message in a received stream.
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
        stream
            .write_all(&self.begin.to_be_bytes())
            .map_err(MessageError::SendingError)?;
        stream
            .write_all(&self.block_length.to_be_bytes())
            .map_err(MessageError::SendingError)?;
        let _ = stream.flush();

        Ok(())
    }
}
