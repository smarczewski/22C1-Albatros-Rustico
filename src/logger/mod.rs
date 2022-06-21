use crate::errors::LoggerError;
//use crate::logger::log_level::LogLevel;
use chrono::DateTime;
use chrono::Local;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
//use std::io::{BufRead}; required for running commented log test

#[derive(Debug)]
pub struct Logger {
    pub file: File,
    pub server_file: File,
}

type PathCheckResult = Result<File, LoggerError>;

impl Logger {
    fn check_filepath_exists(path_archivo: &str) -> PathCheckResult {
        match OpenOptions::new().append(true).open(path_archivo) {
            Ok(file) => Ok(file),
            Err(_error) => Err(LoggerError::FileNotFoundError),
        }
    }

    fn open_client_file_to_be_logged_on(file_path: &str) -> std::io::Result<File> {
        let default_logger_file = "client_log.txt";
        match Logger::check_filepath_exists(file_path) {
            Ok(file) => Ok(file),
            Err(_e) => {
                let concat_path = format!("{}/{}", file_path, default_logger_file);
                let default_file = File::create(concat_path)?;
                Ok(default_file)
            }
        }
    }

    //Opens the file specified in the filePath argument.
    //If invalid, will create a default_file to log on
    fn open_server_file_to_be_logged_on(file_path: &str) -> std::io::Result<File> {
        let default_logger_file = "server_log.txt";
        match Logger::check_filepath_exists(file_path) {
            Ok(file) => Ok(file),
            Err(_e) => {
                let concat_path = format!("{}/{}", file_path, default_logger_file);
                let default_file = File::create(concat_path)?;
                Ok(default_file)
            }
        }
    }

    pub fn logger_create(file_path: &str) -> Result<Logger, LoggerError> {
        if let Ok(file) = Logger::open_client_file_to_be_logged_on(file_path) {
            if let Ok(server_file) = Logger::open_server_file_to_be_logged_on(file_path) {
                let returned_logger = Logger { file, server_file };
                return Ok(returned_logger);
            } else {
                println!("Fallo la creacion del archivo para loggear");
                return Err(LoggerError::FailedToCreateError);
            }
        }
        println!("Fallo la creacion del archivo para loggear");
        Err(LoggerError::FailedToCreateError)
    }

    //writes msje_log to the file previously opened by the logger.
    pub fn log(&mut self, nivel_log: &str, msje_log: &str, sender_type: String) {
        if sender_type == "Server" {
            let _bytes_written = self
                .server_file
                .write(Logger::create_log_record(msje_log, nivel_log).as_bytes());
        } else {
            let _bytes_written = self
                .file
                .write(Logger::create_log_record(msje_log, nivel_log).as_bytes());
        }
    }

    fn create_log_record(msg_to_log: &str, msg_level: &str) -> String {
        let local: DateTime<Local> = Local::now();
        let local = local.format("%Y-%m-%d %H:%M:%S").to_string();
        let debug_level = msg_level.to_string();
        let copy_msg = msg_to_log;
        let junto = format!("[{}]  [{}] {}\n", debug_level, local, copy_msg);
        junto
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_checking_for_non_existent_file_returns_error() {
        let invalid_file = "invalid_file.txt";
        assert!(Result::is_err(&Logger::check_filepath_exists(invalid_file)));
    }

    #[test]
    fn test_checking_for_existent_file_returns_ok() {
        let valid_file = "files_for_testing/settings_files_testing/valid_file.txt";
        assert!(Result::is_ok(&Logger::check_filepath_exists(valid_file)));
    }
}
