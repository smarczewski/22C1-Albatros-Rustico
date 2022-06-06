use crate::errors::MessageError;
use crate::p2p_messages::message_trait::Message;
use std::io::{Read, Write};

#[derive(Debug, PartialEq)]
pub struct PieceMessage {
    _length: u32,
    id: u8,
    piece_index: u32,
    begin: u32,
    block: Vec<u8>,
}

impl PieceMessage {
    /// Create and returns a Piece Message.
    pub fn new(piece_index: u32, begin: u32, block: Vec<u8>) -> Result<PieceMessage, MessageError> {
        if block.is_empty() {
            return Err(MessageError::CreationError);
        }

        Ok(PieceMessage {
            _length: (9 + block.len()) as u32,
            id: 7,
            piece_index,
            begin,
            block,
        })
    }

    /// Reads a Piece Message from a stream and returns the message.
    pub fn read_msg(length: u32, stream: &mut dyn Read) -> Result<PieceMessage, MessageError> {
        let mut buf = [0u8; 4];
        stream
            .read_exact(&mut buf)
            .map_err(MessageError::ReadingError)?;
        let piece_index = u32::from_be_bytes(buf);

        stream
            .read_exact(&mut buf)
            .map_err(MessageError::ReadingError)?;
        let begin = u32::from_be_bytes(buf);

        let mut block = vec![0u8; (length - 9) as usize];
        stream
            .read_exact(&mut block)
            .map_err(MessageError::ReadingError)?;

        PieceMessage::new(piece_index, begin, block)
    }

    pub fn get_piece_index(&self) -> u32 {
        self.piece_index
    }

    pub fn get_begin(&self) -> u32 {
        self.begin
    }

    pub fn get_block(&self) -> Vec<u8> {
        self.block.clone()
    }
}

impl Message for PieceMessage {
    fn print_msg(&self) {
        println!("Type: Piece!\n ID: {}\n", self.id);
        println!(
            "Piece_index: {} , begin: {}, block: {:?}\n",
            self.piece_index, self.begin, self.block
        );
        println!("================================================================\n");
    }

    /// Writes the bytes of a Piece Message in the received stream.
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
            .write_all(&self.block)
            .map_err(MessageError::SendingError)?;
        stream.flush().unwrap();

        Ok(())
    }
}
