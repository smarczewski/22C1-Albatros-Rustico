use crate::parsers::errors::ParseError;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Error};

/// # struct Settings Parser
/// Contains the path of the file to parse.
/// The file has to be in the following format: parameter=value, where
/// the parameters are: 'tcp_port', 'logs_dir_path' and 'download_dir_path'
pub struct SettingsParser<'a>(&'a str);

impl<'a> SettingsParser<'a> {
    /// In case of success, Returns a Hashmap which contains the parameters (as key) and their respective values.
    /// Otherwise, returns a ParseError
    pub fn parse_file(&self) -> Result<HashMap<String, String>, ParseError> {
        if self.0.is_empty() {
            return Err(ParseError::EmptyFilePath);
        }

        let lines = self
            .read_file_lines(self.0)
            .map_err(ParseError::NoSuchFile)?;
        let settings = self.get_settings_from_lines(lines);

        if settings.keys().len() != 3 {
            return Err(ParseError::FileInInvalidFormat);
        }
        Ok(settings)
    }

    /// Receives a file path, reads the file, and returns a vector of strings.
    /// Each element of the vector is a line of the file.
    fn read_file_lines(&self, filename: &str) -> Result<Vec<String>, Error> {
        BufReader::new(File::open(filename)?).lines().collect()
    }

    /// Receives a vector of strings and uses it to create a Hashmap with the settings, which is returned.
    /// Returns the Hashmap.
    fn get_settings_from_lines(&self, lines: Vec<String>) -> HashMap<String, String> {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_file_path() {
        let settings = SettingsParser("").parse_file();
        assert!(settings.is_err());
    }

    #[test]
    fn no_such_file() {
        let settings = SettingsParser("no_such_file.txt").parse_file();
        assert!(settings.is_err());
    }

    #[test]
    fn file_invalid_format_v1() {
        let settings = SettingsParser("settings_files_testing/empty.txt").parse_file();
        assert!(settings.is_err());
    }

    #[test]
    fn file_invalid_format_v2() {
        let settings = SettingsParser("settings_files_testing/invalid_format.txt").parse_file();
        assert!(settings.is_err());
    }

    #[test]
    fn file_valid_format_v1() {
        let received_settings =
            SettingsParser("settings_files_testing/valid_format_v1.txt").parse_file();

        let mut expected_settings = HashMap::new();
        expected_settings.insert("tcp_port".to_string(), "1111".to_string());
        expected_settings.insert("logs_dir_path".to_string(), "/home".to_string());
        expected_settings.insert("download_dir_path".to_string(), "/home".to_string());

        assert_eq!(received_settings.unwrap(), expected_settings);
    }

    #[test]
    fn file_valid_format_v2() {
        let received_settings =
            SettingsParser("settings_files_testing/valid_format_v1.txt").parse_file();

        let mut expected_settings = HashMap::new();
        expected_settings.insert("download_dir_path".to_string(), "/home".to_string());
        expected_settings.insert("tcp_port".to_string(), "1111".to_string());
        expected_settings.insert("logs_dir_path".to_string(), "/home".to_string());

        assert_eq!(received_settings.unwrap(), expected_settings);
    }
}
