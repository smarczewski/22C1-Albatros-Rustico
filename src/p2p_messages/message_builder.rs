use std::io::Read;

use crate::p2p_messages::bitfield::BitfieldMessage;
use crate::p2p_messages::cancel::CancelMessage;
use crate::p2p_messages::choke::ChokeMessage;
use crate::p2p_messages::errors::MessageError;
use crate::p2p_messages::have::HaveMessage;
use crate::p2p_messages::interested::InterestedMessage;
use crate::p2p_messages::keep_alive::KeepAliveMessage;
use crate::p2p_messages::message_trait::Message;
use crate::p2p_messages::not_interested::NotInterestedMessage;
use crate::p2p_messages::piece::PieceMessage;
use crate::p2p_messages::request::RequestMessage;
use crate::p2p_messages::unchoke::UnchokeMessage;

/// # enum P2PMessage
/// Represents the different types of messages in the BitTorrent protocol
pub enum P2PMessage {
    KeepAlive(KeepAliveMessage),
    Choke(ChokeMessage),
    Unchoke(UnchokeMessage),
    Interested(InterestedMessage),
    NotInterested(NotInterestedMessage),
    Have(HaveMessage),
    Bitfield(BitfieldMessage),
    Request(RequestMessage),
    Piece(PieceMessage),
    Cancel(CancelMessage),
}

impl P2PMessage {
    /// Prints a brief description of the message according to its type
    pub fn print_msg(&self) {
        match self {
            P2PMessage::KeepAlive(m) => m.print_msg(),
            P2PMessage::Choke(m) => m.print_msg(),
            P2PMessage::Unchoke(m) => m.print_msg(),
            P2PMessage::Interested(m) => m.print_msg(),
            P2PMessage::NotInterested(m) => m.print_msg(),
            P2PMessage::Have(m) => m.print_msg(),
            P2PMessage::Bitfield(m) => m.print_msg(),
            P2PMessage::Request(m) => m.print_msg(),
            P2PMessage::Piece(m) => m.print_msg(),
            P2PMessage::Cancel(m) => m.print_msg(),
        }
    }
}

/// # struct MessageBuilder
/// It is used to receive a message from a stream
pub struct MessageBuilder;

impl MessageBuilder {
    /// Receives a stream and reads a message from the stream.
    /// On success, returns a P2PMessage enum that contains the message.
    /// Otherwise, returns an error.
    pub fn build(stream: &mut dyn Read) -> Result<P2PMessage, MessageError> {
        let mut len_buf = [0u8; 4];
        stream
            .read_exact(&mut len_buf)
            .map_err(MessageError::ReadingError)?;
        let length = u32::from_be_bytes(len_buf);

        if length == 0 {
            return Ok(P2PMessage::KeepAlive(KeepAliveMessage::new()?));
        }

        let mut id_buf = [0u8; 1];
        stream
            .read_exact(&mut id_buf)
            .map_err(MessageError::ReadingError)?;
        let id = u8::from_be_bytes(id_buf);

        match id {
            0 => Ok(P2PMessage::Choke(ChokeMessage::read_msg(length)?)),
            1 => Ok(P2PMessage::Unchoke(UnchokeMessage::read_msg(length)?)),
            2 => Ok(P2PMessage::Interested(InterestedMessage::read_msg(length)?)),
            3 => Ok(P2PMessage::NotInterested(NotInterestedMessage::read_msg(
                length,
            )?)),
            4 => Ok(P2PMessage::Have(HaveMessage::read_msg(length, stream)?)),
            5 => Ok(P2PMessage::Bitfield(BitfieldMessage::read_msg(
                length, stream,
            )?)),
            6 => Ok(P2PMessage::Request(RequestMessage::read_msg(
                length, stream,
            )?)),
            7 => Ok(P2PMessage::Piece(PieceMessage::read_msg(length, stream)?)),
            8 => Ok(P2PMessage::Cancel(CancelMessage::read_msg(length, stream)?)),
            _ => Err(MessageError::UnknownMessage),
        }
    }
}
