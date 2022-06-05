use super::msg_decoder::MsgDecoder;
use crate::logger::Logger;
use std::sync::mpsc::Receiver;

pub struct LoggerRecvChannel {
    receiver: Receiver<String>,
    logger: Logger,
    counter: i32,
}

impl LoggerRecvChannel {
    //Requires  the logger to be already created
    pub fn new(receiver: Receiver<String>, logger: Logger) -> Self {
        LoggerRecvChannel {
            receiver,
            logger,
            counter: 1,
        }
    }
    fn reduce_counter(&mut self, reduce_value: i32) {
        self.counter -= reduce_value;
    }
    fn increase_counter(&mut self, increase_value: i32) {
        self.counter += increase_value;
    }
    pub fn get_counter(&self) -> i32 {
        //println!("Remaining counter: {}", self.counter);
        self.counter
    }
    pub fn receive(&mut self) -> Result<String, String> {
        if self.counter == 0 {
            Err("Cant receive any more messages".to_string())
        } else {
            self.reduce_counter(1);
            let msg_recv = self.receiver.recv().unwrap();
            let copy_msg = msg_recv;
            let (msg_to_log, increase_value) = MsgDecoder::decode_message(copy_msg);
            self.log(&msg_to_log);
            self.increase_counter(increase_value);
            Ok("Message was logged".to_string())
        }
    }
    //blocking thread function
    fn log(&mut self, message: &str) {
        //AGREGADO PARA METER EL LOG LEVEL
        let msg_copy = message.to_string();
        let log_level = MsgDecoder::get_log_level(msg_copy);
        //FIN DE AGREGADO
        //self.logger.log("ALevel", message);
        self.logger.log(&log_level, message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channel_msg_log::msg_coder::MsgCoder;
    use std::sync::mpsc;
    use std::sync::mpsc::{Receiver, Sender};
    #[test]
    fn test_channel_creates_correctly() {
        let logger = Logger::logger_create("DEBUG", "inexistent_file.txt").unwrap();
        let (_tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();
        let _recv_channel = LoggerRecvChannel::new(rx, logger);
    }

    #[test]
    fn test_channel_reduces_counter_value_correctly() {
        let logger = Logger::logger_create("DEBUG", "inexistent_file.txt").unwrap();
        let (_tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();
        let mut recv_channel = LoggerRecvChannel::new(rx, logger);
        recv_channel.reduce_counter(1);
        assert_eq!(0, recv_channel.get_counter());
    }

    #[test]
    fn test_channel_fails_to_receive_if_counter_equals_zero() {
        let logger = Logger::logger_create("DEBUG", "inexistent_file.txt").unwrap();
        let (_tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();
        let mut recv_channel = LoggerRecvChannel::new(rx, logger);
        recv_channel.reduce_counter(1);
        assert!(Result::is_err(&recv_channel.receive()));
    }

    #[test]
    fn test_channel_reduces_counter_by_one_while_receiving_a_download_finished_message() {
        let logger = Logger::logger_create("DEBUG", "inexistent_file.txt").unwrap();
        let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();
        let mut recv_channel = LoggerRecvChannel::new(rx, logger);
        let mensaje_de_conexion = format!("Mensaje de descarga completa del thread 1");
        let mensaje_a_madar = MsgCoder::generate_download_complete_message(mensaje_de_conexion);
        if tx.send(mensaje_a_madar).is_ok() {
            if recv_channel.receive().is_ok() {
                assert_eq!(0, recv_channel.get_counter());
            } else {
                panic!("");
            }
        } else {
            panic!("");
        }
    }
}
