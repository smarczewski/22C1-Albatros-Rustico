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
    InvalidFormat,
    EmptyVector,
    ReadingFileError(Error),
    IntConvertionError(ParseIntError),
    StrConvertionError(FromUtf8Error),
}

impl ParseError {
    pub fn print_error(&self) {
        match self {
            ParseError::EmptyFilePath => println!("ERROR: The path of the settings file is empty!"),
            ParseError::NoSuchFile(_) => println!("ERROR: No such settings file!"),
            ParseError::InvalidFormat => println!("ERROR: Settings file is in invalid format!"),
            _ => (),
        }
    }
}

#[derive(Debug)]
pub enum RequestError {
    CannotConnectToTracker,
    CannotGetResponse,
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
    InvalidSettings,
    MessageReadingError(MessageError),
    TrackerConnectionError,
    InvalidTrackerResponse,
    CannotFindAnyPeer,
    CannotConnectToPeer,
    ProtocolError,
    StoringPieceError,
}

impl ClientError {
    pub fn print_error(&self) {
        match self {
            ClientError::EmptyTorrentPath => println!("ERROR: The path of torrent file is empty!"),
            ClientError::NoSuchTorrentFile(_) => println!("ERROR: No such torrent file!"),
            ClientError::TorrentInInvalidFormat(_) => {
                println!("ERROR: Torrent file is in invalid format!")
            }
            ClientError::InvalidSettings => println!("ERROR: Client settings are invalid!"),
            ClientError::TrackerConnectionError => {
                println!("ERROR: The tracker connection failed!")
            }
            ClientError::InvalidTrackerResponse => {
                println!("ERROR: The tracker response is invalid!")
            }
            ClientError::CannotFindAnyPeer => {
                println!("ERROR: Cannot find any peer to connect to!")
            }
            _ => (),
        }
    }
}
