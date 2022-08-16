use crate::errors::MessageError;
use std::io::Write;

pub trait Message {
    fn send_msg(&self, stream: &mut dyn Write) -> Result<(), MessageError>;
}
