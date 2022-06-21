pub const TRACKER_PORT: &str = "443";
pub const PEER_ID: &str = "-AR1234-111111111111";
pub const LINES_BEFORE_RES: u8 = 9;

pub const DOWNLOADING: u8 = 1;
pub const NOT_DOWNLOADING: u8 = 0;
pub const CHOKED: u8 = 1;
pub const UNCHOKED: u8 = 0;
pub const INTERESTED: u8 = 1;
pub const NOT_INTERESTED: u8 = 0;

pub const START_LOG_TYPE: u8 = 0;
pub const END_LOG_TYPE: u8 = 1;
pub const ERROR_LOG_TYPE: u8 = 2;
pub const GENERIC_LOG_TYPE: u8 = 3;

pub const CLIENT_MODE_LOG: u8 = 0;
pub const SERVER_MODE_LOG: u8 = 1;
