use std::io::Error;

#[derive(Debug)]
pub enum MessageError {
    ReadingError(Error),
    SendingError(Error),
    CreationError,
    UnknownMessage,
}
