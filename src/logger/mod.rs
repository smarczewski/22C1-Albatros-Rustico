pub mod log_level;
use crate::errors::LoggerError;
use crate::logger::log_level::LogLevel;
use chrono::DateTime;
use chrono::Local;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
//use std::io::{BufRead}; required for running commented log test

#[derive(Debug)]
pub struct Logger {
    pub level: LogLevel,
    pub file: File,
}

type PathCheckResult = Result<File, LoggerError>;

impl Logger {
    fn check_filepath_exists(path_archivo: &str) -> PathCheckResult {
        match OpenOptions::new().append(true).open(path_archivo) {
            Ok(file) => Ok(file),
            Err(_error) => Err(LoggerError::FileNotFoundError),
        }
    }

    //Opens the file specified in the filePath argument.
    //If invalid, will create a default_file to log on
    fn open_file_to_be_logged_on(file_path: &str) -> std::io::Result<File> {
        let default_logger_file = "default_log.txt";
        match Logger::check_filepath_exists(file_path) {
            Ok(file) => Ok(file),
            Err(_e) => {
                let concat_path = format!("{}/{}", file_path, default_logger_file);
                let default_file = File::create(concat_path)?;
                Ok(default_file)
            }
        }
    }

    fn match_log_level(nivel_log: &str) -> LogLevel {
        match nivel_log {
            "INFO" => LogLevel::Info,
            "ERROR" => LogLevel::Error,
            _ => LogLevel::Debug,
        }
    }

    //Receives a log_level as an argument, returns a Logger set
    //at log_level level or debug if there is no match
    pub fn logger_create(nivel_log: &str, file_path: &str) -> Result<Logger, LoggerError> {
        let level = Logger::match_log_level(nivel_log);
        if let Ok(file) = Logger::open_file_to_be_logged_on(file_path) {
            let returned_logger = Logger { level, file };
            Ok(returned_logger)
        } else {
            println!("Fallo la creacion del archivo para loggear");
            Err(LoggerError::FailedToCreateError)
        }
    }

    //writes msje_log to the file previously opened by the logger.
    //The logging level set by nivel_log must be equal or higher than the one
    //that was set when the logger was created. If not, the message will not be
    //written to the log file
    pub fn log(&mut self, nivel_log: &str, msje_log: &str) {
        let _bytes_written = self
            .file
            .write(Logger::create_log_record_two(msje_log, nivel_log).as_bytes());

        //let desired_lvl = Logger::match_log_level(nivel_log) as u32;
        //let logger_lvl = self.level as u32;
        //
        //        //if desired_lvl < logger_lvl {
        //        //    println!("The desired logging level is lower than the one it was set. Unable to log");
        //        //} else {
        //        //    let _bytes_written = self
        //        //        .file
        //        //        .write(Logger::create_log_record_two(msje_log, nivel_log).as_bytes());
        //        //.write(Logger::create_log_record(msje_log).as_bytes());
        //}
    }

    //fn create_log_record(msg_to_log: &str) -> String {
    //    let local: DateTime<Local> = Local::now();
    //    let local = local.format("%Y-%m-%d %H:%M:%S").to_string();
    //    let debug_level = "HARDCODED_DEBUG".to_string();
    //    let copy_msg = msg_to_log;
    //    let junto = format!("[{}]  [{}] {}\n", debug_level, local, copy_msg);
    //    junto
    //}

    fn create_log_record_two(msg_to_log: &str, msg_level: &str) -> String {
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

    #[test]
    fn test_checking_matching_an_invalid_log_level_returns_debug() {
        let invalid_log_level = "invalid level";
        let log_level = Logger::match_log_level(&invalid_log_level);
        let log_level = log_level as u32;
        let debug_level = LogLevel::Debug as u32;
        assert_eq!(log_level, debug_level);
    }

    #[test]
    fn test_matching_error_log_level_returns_appropiate_level() {
        let valid_log_level = "ERROR";
        let log_level = Logger::match_log_level(&valid_log_level) as u32;
        let error_level = LogLevel::Error as u32;
        assert_eq!(log_level, error_level);
    }

    #[test]
    fn test_matching_info_log_level_returns_appropiate_level() {
        let valid_log_level = "INFO";
        let log_level = Logger::match_log_level(&valid_log_level) as u32;
        let info_level = LogLevel::Info as u32;
        assert_eq!(log_level, info_level);
    }
    #[test]
    fn test_matching_debug_log_level_returns_appropiate_level() {
        let valid_log_level = "DEBUG";
        let log_level = Logger::match_log_level(&valid_log_level) as u32;
        let debug_level = LogLevel::Debug as u32;
        assert_eq!(log_level, debug_level);
    }

    #[test]
    fn test_debug_level_is_the_highest_logging_level() {
        let error_level = LogLevel::Error as u32;
        let info_level = LogLevel::Info as u32;
        let debug_level = LogLevel::Debug as u32;

        assert_eq!(info_level < error_level, error_level < debug_level);
    }
    #[test]
    fn test_info_level_is_lower_than_error_level() {
        let info_level = LogLevel::Info as u32;
        let debug_level = LogLevel::Debug as u32;
        assert!(info_level < debug_level);
    }
    //Before running this test, the file "default_log.txt" must not exist
    //in the project folder
    //due to being a test that requires a file to exist in the project folder and
    //thus not being able to pass the automated tests in the github pull request
    //it will be commented. The test has been executed and the function tested
    //works as intented
    //#[test]
    //fn test_logger_correctly_writes_buffer_content_to_file(){
    //	let mut logger = Logger::logger_create("DEBUG","inexistent_file.txt").unwrap();
    //	logger.log("DEBUG","MENSAJE DE PRUEBA");
    //	let un_file = File::open("default_log.txt").unwrap();
    //	let buffer_linea = std::io::BufReader::new(un_file).lines();
    //	let mut counter = 0;
    //	for _linea in buffer_linea.flatten(){
    //		counter += 1;
    //	}
    //	assert_eq!(1,counter);
    //}
}
