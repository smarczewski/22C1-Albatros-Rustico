use crate::errors::ArgsError;
use std::collections::HashMap;
use std::fs::File;
use std::io::Error;
use std::io::{BufRead, BufReader};

/// # struct Settings
/// - log_dir -> Path of the directory where the log file will be
/// - tcp_port -> Port on which we will listen for connections
/// - downloads_dir -> Path of the directory where the downloaded files will be
#[derive(Debug, Clone)]
pub struct Settings {
    log_dir: String,
    tcp_port: String,
    downloads_dir: String,
}

impl Settings {
    /// Receives the path of the settings file, parses it, and creates a Settings struct.
    /// On success, returns successfully initialized settings.
    /// Other wise, returns an error.
    pub fn new(file_path: &str) -> Result<Settings, ArgsError> {
        if file_path.is_empty() {
            return Err(ArgsError::EmptySettingsPath);
        }

        let settings_dict = Settings::parse_file(file_path)?;

        let downloads = settings_dict.get(&"download_dir_path".to_string());
        let port = settings_dict.get(&"tcp_port".to_string());
        let log = settings_dict.get(&"logs_dir_path".to_string());

        if let (Some(downloads_dir), Some(tcp_port), Some(log_dir)) = (downloads, port, log) {
            return Ok(Settings {
                log_dir: log_dir.to_string(),
                tcp_port: tcp_port.to_string(),
                downloads_dir: downloads_dir.to_string(),
            });
        }

        Err(ArgsError::InvalidSettings)
    }

    /// Parses settings file.
    /// In case of success, Returns a Hashmap which contains the parameters (as key) and their respective values.
    /// Otherwise, returns a ParseError
    fn parse_file(file_path: &str) -> Result<HashMap<String, String>, ArgsError> {
        if let Ok(lines) = Settings::read_file_lines(file_path) {
            let settings = Settings::get_settings_from_lines(lines);

            return Ok(settings);
        }

        Err(ArgsError::NoSuchSettingsFile)
    }

    /// Receives a file path, reads the file, and returns a vector of strings.
    /// Each element of the vector is a line of the file.
    fn read_file_lines(filename: &str) -> Result<Vec<String>, Error> {
        BufReader::new(File::open(filename)?).lines().collect()
    }

    /// Receives a vector of strings and uses it to create a Hashmap with the settings, which is returned.
    /// Returns the Hashmap.
    fn get_settings_from_lines(lines: Vec<String>) -> HashMap<String, String> {
        let mut settings = HashMap::new();

        for line in lines {
            let mut split_line = line.split('=');

            let key = split_line.next();
            let value = split_line.next();

            if let (Some(k), Some(v)) = (key, value) {
                if k == "tcp_port" || k == "logs_dir_path" || k == "download_dir_path" {
                    settings.insert(k.to_string(), v.to_string());
                }
            }
        }
        settings
    }

    pub fn get_log_dir(&self) -> String {
        self.log_dir.clone()
    }

    pub fn get_tcp_port(&self) -> String {
        self.tcp_port.clone()
    }

    pub fn get_downloads_dir(&self) -> String {
        self.downloads_dir.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_file_path() {
        let settings = Settings::new("");
        match settings {
            Err(ArgsError::EmptySettingsPath) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn no_such_file() {
        let settings = Settings::new("no_such_file.txt");
        match settings {
            Err(ArgsError::NoSuchSettingsFile) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn file_invalid_format_v1() {
        let settings = Settings::new("files_for_testing/settings_files_testing/empty.txt");
        match settings {
            Err(ArgsError::InvalidSettings) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn file_invalid_format_v2() {
        let settings = Settings::new("files_for_testing/settings_files_testing/invalid_format.txt");
        match settings {
            Err(ArgsError::InvalidSettings) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn file_valid_format_v1() {
        let path = "files_for_testing/settings_files_testing/valid_format_v1.txt";
        if let Ok(received_settings) = Settings::new(path) {
            let l = received_settings.get_log_dir();
            let p = received_settings.get_tcp_port();
            let d = received_settings.get_downloads_dir();

            assert_eq!(l, "log");
            assert_eq!(p, "8080");
            assert_eq!(d, "downloaded_files");
        } else {
            assert!(false);
        }
    }

    #[test]
    fn file_valid_format_v2() {
        let path = "files_for_testing/settings_files_testing/valid_format_v2.txt";
        if let Ok(received_settings) = Settings::new(path) {
            let l = received_settings.get_log_dir();
            let p = received_settings.get_tcp_port();
            let d = received_settings.get_downloads_dir();

            assert_eq!(l, "/home");
            assert_eq!(p, "8080");
            assert_eq!(d, "downloaded_files");
        } else {
            assert!(false);
        }
    }
}
