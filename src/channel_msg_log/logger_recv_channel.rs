use super::msg_decoder::MsgDecoder;
use crate::logger::Logger;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
#[derive(Debug)]
pub struct LoggerRecvChannel {
    receiver: Receiver<String>,
    logger: Logger,
    number_of_end_connections: u8,
}

impl LoggerRecvChannel {
    //Takes the path where the logger files are to be created.
    //Creates the communication channels and the logger
    pub fn new(
        file_path: &str,
        num_end_con: u8,
    ) -> Result<(Sender<String>, LoggerRecvChannel), String> {
        if let Ok(logger) = Logger::logger_create(file_path) {
            let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();
            let channel = LoggerRecvChannel {
                receiver: rx,
                logger,
                number_of_end_connections: num_end_con,
            };
            Ok((tx, channel))
        } else {
            Err("Failed to create logger".to_string())
        }
    }

    pub fn continue_receiving(&mut self) -> bool {
        self.number_of_end_connections > 0
    }

    fn update_end_messages_received(&mut self, continue_status: bool) {
        if !continue_status {
            self.number_of_end_connections -= 1;
        }
    }
    pub fn receive(&mut self) -> Result<String, String> {
        let msg_recv = self.receiver.recv().unwrap();
        let copy_msg = msg_recv;
        let (sender_type, continue_receiving_status, message, message_type) =
            MsgDecoder::decypher_message(copy_msg);
        self.update_end_messages_received(continue_receiving_status);
        self.log(sender_type, &message, &message_type);
        Ok("Message was logged".to_string())
    }
    //blocking thread function
    fn log(&mut self, sender_type: String, message: &str, message_type: &str) {
        //let msg_copy = message.to_string();
        self.logger.log(message_type, message, sender_type);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channel_msg_log::msg_coder::MsgCoder;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::mpsc;
    use std::sync::mpsc::{Receiver, Sender};
    #[test]
    #[test]
    fn test_channel_creates_correctly() {
        let srcdir = PathBuf::from("./files_for_testing");
        let src_dir = fs::canonicalize(&srcdir).unwrap();
        let abs_path = format!("{}/", src_dir.as_path().display().to_string());
        let channel_touple = LoggerRecvChannel::new(&abs_path, 2);
        assert!(channel_touple.is_ok());
    }
}
