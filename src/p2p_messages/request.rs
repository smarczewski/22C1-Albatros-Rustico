use crate::errors::MessageError;
use crate::p2p_messages::message_trait::Message;
use std::io::{Read, Write};

#[derive(Debug, PartialEq)]
pub struct RequestMessage {
    _length: u32,
    id: u8,
    piece_index: u32,
    begin: u32,
    block_length: u32,
}

impl RequestMessage {
    /// Create and returns a Request Message.
    pub fn new(
        piece_index: u32,
        begin: u32,
        block_length: u32,
    ) -> Result<RequestMessage, MessageError> {
        if block_length == 0 {
            return Err(MessageError::CreationError);
        }

        Ok(RequestMessage {
            _length: 13,
            id: 6,
            piece_index,
            begin,
            block_length,
        })
    }

    /// Reads a Request Message from a stream and returns the message.
    pub fn read_msg(length: u32, stream: &mut dyn Read) -> Result<RequestMessage, MessageError> {
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

        RequestMessage::new(piece_index, begin, block_length)
    }
}

impl Message for RequestMessage {
    fn print_msg(&self) {
        println!("Type: Request!\n ID: {}\n", self.id);
        println!(
            "Piece index: {}, begin: {}, block_length: {}\n",
            self.piece_index, self.begin, self.block_length
        );
        println!("================================================================\n");
    }

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
        stream.flush().unwrap();

        Ok(())
    }
}
