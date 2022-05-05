use std::io::Error;

#[derive(Debug)]
pub enum ParseError {
    EmptyFilePath,
    NoSuchFile(Error),
    FileInInvalidFormat,
}
