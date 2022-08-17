use crate::constants::*;
pub struct MsgCoder {}

impl MsgCoder {
    //message type: START for signaling the start of logging
    //end type: END for signaling the start of
    pub fn generate_message(message_type: u8, sender_mode: u8, message: String) -> String {
        let emiter = match sender_mode {
            SERVER_MODE_LOG => "Server",
            _ => "Client",
        };
        match message_type {
            START_LOG_TYPE => format!("START|{}|{}", emiter, message),
            END_LOG_TYPE => format!("END|{}|{}", emiter, message),
            ERROR_LOG_TYPE => format!("ERROR|{}|{}", emiter, message),
            _ => format!("INFO|{}|{}", emiter, message),
        }
    }
}
