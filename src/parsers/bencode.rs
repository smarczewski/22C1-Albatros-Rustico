use crate::parsers::errors::ParseError;
use std::collections::HashMap;
use std::fs::File;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Read;
use std::string::String;

/// # enum Bencode Type
/// Represents the four different data types supported by the Bencode format.
/// Also, one more type is End, which indicates the end of a structure.
#[derive(Debug, PartialEq)]
pub enum BencodeType {
    String(Vec<u8>),
    Integer(i64),
    List(Vec<BencodeType>),
    Dictionary(HashMap<String, BencodeType>),
    End,
}

/// # struct Bencode Parser
/// Its only pub function is parse_file(), which receives the file to parse.
/// The file has to be in the bencode format
/// Supported data types are: integers, strings, lists, and dictionaries.
pub struct BencodeParser;

impl BencodeParser {
    /// On success, returns a BencodeType enum which contains the parsed element.
    /// Otherwise, returns ParseError.
    pub fn parse_file(&self, path: &str) -> Result<BencodeType, ParseError> {
        if path.is_empty() {
            return Err(ParseError::EmptyFilePath);
        }
        let mut file = File::open(path).map_err(ParseError::NoSuchFile)?;
        self.parse(&mut file)
    }

    ///Reads a byte from the file, and then decides what to do according to the byte reading.
    /// If the byte is a 'd', it proceeds to read a dictionary.
    /// If the byte is a 'l', it proceeds to read a list.
    /// If the byte is a 'i', it proceeds to read an integer.
    /// If the byte is a numeric char, it proceeds to read a string.
    /// Otherwise, returns ParseError (invalid format)  
    fn parse(&self, file: &mut File) -> Result<BencodeType, ParseError> {
        let mut current_byte = [0u8; 1];
        file.read_exact(&mut current_byte)
            .map_err(ParseError::FileInInvalidFormat)?;

        let current_char = current_byte[0] as char;
        match current_char {
            'd' => self.read_dictionary(file),
            'l' => self.read_list(file),
            'i' => self.read_integer(file),
            'e' => Ok(BencodeType::End),
            _ if current_char.is_numeric() => self.read_string(current_char, file),
            _ => Err(self.invalid_format_error()),
        }
    }

    /// Reads a bencoded string from the file.
    /// On success, returns BencodeType::String that contains the string as vec<u8>
    /// Otherwise, returns ParseError.
    fn read_string(&self, first_char: char, file: &mut File) -> Result<BencodeType, ParseError> {
        let mut length_aux = String::new();
        length_aux.push(first_char);

        loop {
            let mut current_byte = [0u8; 1];
            file.read_exact(&mut current_byte)
                .map_err(ParseError::FileInInvalidFormat)?;
            let current_char = current_byte[0] as char;
            if current_char == ':' {
                break;
            }
            length_aux.push(current_char);
        }

        let length = length_aux
            .parse::<u32>()
            .map_err(ParseError::IntConvertionError)?;

        let mut buf = vec![0; length as usize];
        file.read_exact(&mut buf)
            .map_err(ParseError::FileInInvalidFormat)?;
        Ok(BencodeType::String(buf))
    }

    /// Reads a bencoded integer from the file.
    /// On success, returns BencodeType::Integer that contains the integer as i64
    /// Otherwise, returns ParseError.
    fn read_integer(&self, file: &mut File) -> Result<BencodeType, ParseError> {
        let mut integer_aux = String::new();
        loop {
            let mut current_byte = [0u8; 1];
            file.read_exact(&mut current_byte)
                .map_err(ParseError::FileInInvalidFormat)?;

            let current_char = current_byte[0] as char;
            match current_char {
                'e' => break,
                _ if current_char.is_numeric() => {
                    integer_aux.push(current_char);
                }
                _ => return Err(self.invalid_format_error()),
            }
        }

        if integer_aux.is_empty() {
            return Err(self.invalid_format_error());
        }

        let integer = integer_aux
            .parse::<i64>()
            .map_err(ParseError::IntConvertionError)?;
        Ok(BencodeType::Integer(integer))
    }

    /// Reads a bencoded list from the file.
    /// On success, returns BencodeType::List that contains the list as vec<BencodeType>
    /// Otherwise, returns ParseError.
    fn read_list(&self, file: &mut File) -> Result<BencodeType, ParseError> {
        let mut list = Vec::<BencodeType>::new();

        loop {
            let current_element = self.parse(file)?;
            if let BencodeType::End = current_element {
                break;
            }
            list.push(current_element);
        }

        if list.is_empty() {
            return Err(self.invalid_format_error());
        }

        Ok(BencodeType::List(list))
    }

    /// Reads a bencoded dictionary from the file.
    /// On success, returns BencodeType::Dictionary that contains the dictionary as HashMap<String,BencodeType>
    /// Otherwise, returns ParseError.
    fn read_dictionary(&self, file: &mut File) -> Result<BencodeType, ParseError> {
        let mut dic = HashMap::new();

        loop {
            let key_aux = self.parse(file)?;
            if let BencodeType::End = key_aux {
                break;
            }

            let value = self.parse(file)?;
            let key: String = match (key_aux, &value) {
                (_, BencodeType::End) => {
                    return Err(self.invalid_format_error());
                }
                (BencodeType::String(s), _) => {
                    String::from_utf8(s).map_err(ParseError::StrConvertionError)?
                }
                _ => {
                    return Err(self.invalid_format_error());
                }
            };
            dic.insert(key, value);
        }
        if dic.is_empty() {
            return Err(self.invalid_format_error());
        }

        Ok(BencodeType::Dictionary(dic))
    }

    /// Creates a FileInInvalidFormat error, and returns it.
    fn invalid_format_error(&self) -> ParseError {
        ParseError::FileInInvalidFormat(Error::new(ErrorKind::InvalidData, "Invalid format"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_file_path() {
        let integer_parsed = BencodeParser.parse_file("");
        match integer_parsed {
            Err(ParseError::EmptyFilePath) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn no_such_file() {
        let integer_parsed = BencodeParser.parse_file("bencoded_files_testing/not_exist.txt");
        match integer_parsed {
            Err(ParseError::NoSuchFile(_)) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn file_empty() {
        let integer_parsed = BencodeParser.parse_file("bencoded_files_testing/empty.txt");
        match integer_parsed {
            Err(ParseError::FileInInvalidFormat(_)) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn file_invalid_format() {
        let integer_parsed = BencodeParser.parse_file("bencoded_files_testing/invalid_format.txt");
        match integer_parsed {
            Err(ParseError::FileInInvalidFormat(_)) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn reading_integer() {
        let integer_parsed = BencodeParser.parse_file("bencoded_files_testing/integer.txt");
        let expected_value = BencodeType::Integer(12345);
        assert_eq!(integer_parsed.unwrap(), expected_value);
    }

    #[test]
    fn reading_string() {
        let integer_parsed = BencodeParser.parse_file("bencoded_files_testing/string.txt");
        let expected_value = BencodeType::String("hello bittorrent".as_bytes().to_vec());
        assert_eq!(integer_parsed.unwrap(), expected_value);
    }

    #[test]
    fn reading_list() {
        let integer_parsed = BencodeParser.parse_file("bencoded_files_testing/list.txt");
        let list = vec![
            BencodeType::Integer(1),
            BencodeType::Integer(2),
            BencodeType::Integer(3),
        ];
        let expected_value = BencodeType::List(list);
        assert_eq!(integer_parsed.unwrap(), expected_value);
    }

    #[test]
    fn reading_dictionary() {
        let integer_parsed = BencodeParser.parse_file("bencoded_files_testing/dictionary.txt");
        let mut dic = HashMap::new();
        dic.insert(
            "announce".to_string(),
            BencodeType::String("https://torrent.ubuntu.com/announce".as_bytes().to_vec()),
        );
        dic.insert(
            "created by".to_string(),
            BencodeType::String("mktorrent 1.1".as_bytes().to_vec()),
        );
        dic.insert(
            "creation date".to_string(),
            BencodeType::Integer(1645734650),
        );

        let mut info = HashMap::new();
        info.insert("length".to_string(), BencodeType::Integer(3379068928));
        info.insert(
            "name".to_string(),
            BencodeType::String("ubuntu-20.04.4-desktop-amd64.iso".as_bytes().to_vec()),
        );
        info.insert("piece length".to_string(), BencodeType::Integer(262144));
        info.insert(
            "pieces".to_string(),
            BencodeType::String("xyz".as_bytes().to_vec()),
        );

        dic.insert("info".to_string(), BencodeType::Dictionary(info));

        let expected_value = BencodeType::Dictionary(dic);
        assert_eq!(integer_parsed.unwrap(), expected_value);
    }
}
