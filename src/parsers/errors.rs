use std::io::Error;
use std::num::ParseIntError;
use std::string::FromUtf8Error;

#[derive(Debug)]
pub enum ParseError {
    EmptyFilePath,
    NoSuchFile(Error),
    FileInInvalidFormat,
    FileReadingError(Error),
    IntConvertionError(ParseIntError),
    StrConvertionError(FromUtf8Error),
}
