use super::msg_decoder::MsgDecoder;
use crate::errors::LoggerError;
use crate::logger::Logger;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
#[derive(Debug)]
pub struct LoggerRecvChannel {
    receiver: Receiver<String>,
    logger: Logger,
    continue_receiving: bool,
}

impl LoggerRecvChannel {
    //Takes the path where the logger files are to be created.
    //Creates the communication channels and the logger
    pub fn new(file_path: &str) -> Result<(Sender<String>, LoggerRecvChannel), LoggerError> {
        let logger = Logger::logger_create(file_path)?;
        let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();
        let channel = LoggerRecvChannel {
            receiver: rx,
            logger,
            continue_receiving: true,
        };
        Ok((tx, channel))
    }

    pub fn continue_receiving(&mut self) -> bool {
        self.continue_receiving
    }

    fn update_end_messages_received(&mut self, continue_status: bool) {
        self.continue_receiving = continue_status;
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
    use std::fs;
    use std::path::PathBuf;
    #[test]
    fn test_channel_creates_correctly() {
        let srcdir = PathBuf::from("./files_for_testing");
        let src_dir = fs::canonicalize(&srcdir).unwrap();
        let abs_path = format!("{}/", src_dir.as_path().display().to_string());
        let channel_touple = LoggerRecvChannel::new(&abs_path);
        assert!(channel_touple.is_ok());
    }
}
