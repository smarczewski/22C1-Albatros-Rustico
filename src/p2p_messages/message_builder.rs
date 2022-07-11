use std::io::Read;

use crate::errors::MessageError;
use crate::p2p_messages::bitfield::BitfieldMsg;
use crate::p2p_messages::cancel::CancelMsg;
use crate::p2p_messages::choke::ChokeMsg;
use crate::p2p_messages::have::HaveMsg;
use crate::p2p_messages::interested::InterestedMsg;
use crate::p2p_messages::keep_alive::KeepAliveMsg;
use crate::p2p_messages::not_interested::NotInterestedMsg;
use crate::p2p_messages::piece::PieceMsg;
use crate::p2p_messages::request::RequestMsg;
use crate::p2p_messages::unchoke::UnchokeMsg;

/// # enum P2PMessage
/// Represents the different types of messages in the BitTorrent protocol
pub enum P2PMessage {
    KeepAlive(KeepAliveMsg),
    Choke(ChokeMsg),
    Unchoke(UnchokeMsg),
    Interested(InterestedMsg),
    NotInterested(NotInterestedMsg),
    Have(HaveMsg),
    Bitfield(BitfieldMsg),
    Request(RequestMsg),
    Piece(PieceMsg),
    Cancel(CancelMsg),
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
            return Ok(P2PMessage::KeepAlive(KeepAliveMsg::new()));
        }

        let mut id_buf = [0u8; 1];
        stream
            .read_exact(&mut id_buf)
            .map_err(MessageError::ReadingError)?;
        let id = u8::from_be_bytes(id_buf);
        match id {
            0 => Ok(P2PMessage::Choke(ChokeMsg::read_msg(length)?)),
            1 => Ok(P2PMessage::Unchoke(UnchokeMsg::read_msg(length)?)),
            2 => Ok(P2PMessage::Interested(InterestedMsg::read_msg(length)?)),
            3 => Ok(P2PMessage::NotInterested(NotInterestedMsg::read_msg(
                length,
            )?)),
            4 => Ok(P2PMessage::Have(HaveMsg::read_msg(length, stream)?)),
            5 => Ok(P2PMessage::Bitfield(BitfieldMsg::read_msg(length, stream)?)),
            6 => Ok(P2PMessage::Request(RequestMsg::read_msg(length, stream)?)),
            7 => Ok(P2PMessage::Piece(PieceMsg::read_msg(length, stream)?)),
            8 => Ok(P2PMessage::Cancel(CancelMsg::read_msg(length, stream)?)),
            _ => Err(MessageError::UnknownMessage),
        }
    }
}
