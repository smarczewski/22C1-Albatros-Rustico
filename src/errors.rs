use native_tls::Error as TlsError;
use std::io::Error;
use std::num::ParseIntError;
use std::string::FromUtf8Error;

#[derive(Debug)]
pub enum TypeError {
    IsNotDictionary,
    IsNotString,
    IsNotInteger,
    IsNotList,
}

#[derive(Debug)]
pub enum ParseError {
    EmptyFilePath,
    NoSuchFile(Error),
    ReadingFileError(Error),
    EmptyVector,
    InvalidFormat,
    IntConvertionError(ParseIntError),
    StrConvertionError(FromUtf8Error),
}

#[derive(Debug)]
pub enum RequestError {
    TorrentInInvalidFormat(TypeError),
    StrConvertionError(FromUtf8Error),
    TlsConnectionError(TlsError),
    CannotGetResponse,
    ParserError(ParseError),
    InvalidResponse,
}

#[derive(Debug)]
pub enum MessageError {
    ReadingError(Error),
    SendingError(Error),
    CreationError,
    UnknownMessage,
}

#[derive(Debug)]
pub enum LoggerError {
    FileNotFoundError,
    FailedToCreateError,
}

#[derive(Debug)]
pub enum ClientError {
    EmptyTorrentPath,
    NoSuchTorrentFile(ParseError),
    TorrentInInvalidFormat(TypeError),
    StrConvertionError(FromUtf8Error),
    InvalidSettings,
    MessageReadingError(MessageError),
    TrackerConnectionError,
    InvalidTrackerResponse,
    CannotConnectToPeer,
    ProtocolError,
    StoringPieceError,
}
