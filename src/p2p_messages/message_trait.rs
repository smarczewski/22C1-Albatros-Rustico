use crate::p2p_messages::errors::MessageError;
use std::io::Write;

pub trait Message {
    fn print_msg(&self);
    fn send_msg(&self, stream: &mut dyn Write) -> Result<(), MessageError>;
}
