use std::io::Error;
use std::num::ParseIntError;
use std::process::exit;
use std::string::FromUtf8Error;

pub trait ErrorMessage {
    fn print_error(&self);
}

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

#[derive(Debug)]
pub enum ArgsError {
    EmptySettingsPath,
    NoSuchSettingsFile,
    InvalidSettings,
    NoTorrentDir,
}
impl ErrorMessage for ArgsError {
    fn print_error(&self) {
        match self {
            ArgsError::EmptySettingsPath => {
                println!("ERROR: The path of the settings file is empty!")
            }
            ArgsError::NoSuchSettingsFile => println!("ERROR: No such settings file!"),
            ArgsError::InvalidSettings => {
                println!("ERROR: Settings file is in invalid format!")
            }
            ArgsError::NoTorrentDir => println!("ERROR: Cannot find the torrents directory!"),
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

impl ErrorMessage for LoggerError {
    fn print_error(&self) {
        if let LoggerError::FailedToCreateError = self {
            println!("ERROR: Cannot create logger file!")
        };
    }
}

pub enum DownloadError {
    ConnectionFailed,
    NoWantedPieces,
    PeerHasNotThePiece,
    PeerChokedUs,
    InvalidPiece,
    CannotReadPeerMessage,
    ConnectionFinished,
    NoPeers,
    HandshakeError,
}

#[derive(Debug)]
pub enum ClientError {
    EmptyTorrentPath,
    NoSuchTorrentFile(ParseError),
    TorrentInInvalidFormat(TypeError),
    CannotFindDownloadDir,
    InvalidSettings,
    MessageReadingError(MessageError),
    TrackerConnectionError,
    InvalidTrackerResponse,
    CannotFindAnyPeer,
    CannotConnectToPeer,
    ProtocolError,
    StoringPieceError,
    DownloadError,
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

pub enum ServerError {
    HandshakeError,
    CannotFindTorrent,
    CannotReadPeerMessage,
    NoSuchDirectory,
    PieceError,
}

pub trait HandleError<T> {
    fn handle_error(self) -> T;
}

impl<T, E: ErrorMessage> HandleError<T> for Result<T, E> {
    fn handle_error(self) -> T {
        match self {
            Ok(v) => v,
            Err(e) => {
                e.print_error();
                exit(-1);
            }
        }
    }
}
