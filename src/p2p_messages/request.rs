use crate::errors::MessageError;
use crate::p2p_messages::message_trait::Message;
use std::io::{Read, Write};

#[derive(Debug, PartialEq)]
pub struct RequestMsg {
    _length: u32,
    id: u8,
    piece_index: u32,
    begin: u32,
    block_length: u32,
}

impl RequestMsg {
    /// Create and returns a Request Message.
    pub fn new(
        piece_index: u32,
        begin: u32,
        block_length: u32,
    ) -> Result<RequestMsg, MessageError> {
        if block_length == 0 {
            return Err(MessageError::CreationError);
        }

        Ok(RequestMsg {
            _length: 13,
            id: 6,
            piece_index,
            begin,
            block_length,
        })
    }

    /// Reads a Request Message from a stream and returns the message.
    pub fn read_msg(length: u32, stream: &mut dyn Read) -> Result<RequestMsg, MessageError> {
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

        RequestMsg::new(piece_index, begin, block_length)
    }

    pub fn get_piece_index(&self) -> u32 {
        self.piece_index
    }

    pub fn get_begin(&self) -> u32 {
        self.begin
    }

    pub fn get_block_length(&self) -> u32 {
        self.block_length
    }
}

impl Message for RequestMsg {
    /// Writes the bytes of a Request Message in the received stream.
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
