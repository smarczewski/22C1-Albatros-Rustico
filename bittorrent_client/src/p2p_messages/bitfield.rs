use crate::errors::MessageError;
use crate::p2p_messages::message_trait::Message;
use std::io::{Read, Write};

#[derive(Debug, PartialEq, Eq)]
pub struct BitfieldMsg {
    _length: u32,
    id: u8,
    pieces: Vec<u8>,
}

impl BitfieldMsg {
    /// Create and returns a Bitfield Message.
    pub fn new(pieces: Vec<u8>) -> Result<BitfieldMsg, MessageError> {
        if pieces.is_empty() {
            return Err(MessageError::CreationError);
        }

        Ok(BitfieldMsg {
            _length: (1 + pieces.len()) as u32,
            id: 5,
            pieces,
        })
    }

    /// Reads a Bitfield Message from a stream and returns the message.
    pub fn read_msg(length: u32, stream: &mut dyn Read) -> Result<BitfieldMsg, MessageError> {
        let mut pieces = vec![0u8; (length - 1) as usize];
        stream
            .read_exact(&mut pieces)
            .map_err(MessageError::ReadingError)?;

        BitfieldMsg::new(pieces)
    }

    /// Returns vector of pieces
    pub fn get_pieces(&self) -> Vec<u8> {
        self.pieces.clone()
    }
}

impl Message for BitfieldMsg {
    /// Writes the bytes of a Bitfield Message in a received stream.
    fn send_msg(&self, stream: &mut dyn Write) -> Result<(), MessageError> {
        stream
            .write_all(&self._length.to_be_bytes())
            .map_err(MessageError::SendingError)?;
        stream
            .write_all(&self.id.to_be_bytes())
            .map_err(MessageError::SendingError)?;
        stream
            .write_all(&self.pieces)
            .map_err(MessageError::SendingError)?;
        let _ = stream.flush();

        Ok(())
    }
}
