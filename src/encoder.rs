use crate::parsers::bencode::BencodeType;
use std::collections::HashMap;

/// # struct Encoder
/// Supports two formats: Bencode and URLencode.
pub struct Encoder;

impl Encoder {
    /// Receives a byte slice, encodes it with URLencode format and returns it as string.
    pub fn urlencode(&self, vec: &[u8]) -> String {
        let mut urlencoded_data = String::new();

        for curr_byte in vec {
            let curr_char = *curr_byte as char;
            match curr_char {
                '0'..='9' | 'a'..='z' | 'A'..='Z' | '.' | '-' | '_' | '~' => {
                    urlencoded_data.push(curr_char);
                }
                _ => {
                    let byte_hex = format!("{:X}", curr_byte);
                    if byte_hex.len() < 2 {
                        urlencoded_data = format!("{}%0{}", urlencoded_data, byte_hex);
                    } else {
                        urlencoded_data = format!("{}%{}", urlencoded_data, byte_hex);
                    }
                }
            }
        }
        urlencoded_data
    }

    /// Receives a decoded BencodeType element and encodes it with Bencode format.
    /// Then, returns it as vec<u8>
    pub fn bencode(&self, decoded: &BencodeType) -> Vec<u8> {
        let mut bencoded_data = Vec::<u8>::new();
        self.bencode_type(decoded, &mut bencoded_data);
        bencoded_data
    }

    /// Receives a BencodeType element and a vec<u8>, and adds the encoded element to the vec
    fn bencode_type(&self, ben_type: &BencodeType, vec: &mut Vec<u8>) {
        match ben_type {
            BencodeType::String(s) => self.bencode_string(s, vec),
            BencodeType::Integer(i) => self.bencode_integer(*i, vec),
            BencodeType::List(l) => self.bencode_list(l.as_slice(), vec),
            BencodeType::Dictionary(d) => self.bencode_dictionary(d, vec),
            _ => (),
        }
    }

    /// Encodes an integer with Bencode format
    fn bencode_integer(&self, integer: i64, vec: &mut Vec<u8>) {
        vec.push(b'i');

        let int_str = integer.to_string();
        vec.append(&mut int_str.into_bytes());
        vec.push(b'e');
    }

    /// Encodes a string with Bencode format
    fn bencode_string(&self, string: &[u8], vec: &mut Vec<u8>) {
        let mut length = string.len().to_string();
        length.push(':');
        *vec = [vec, length.as_bytes(), string].concat();
    }

    /// Encodes a list with Bencode format
    fn bencode_list(&self, list: &[BencodeType], vec: &mut Vec<u8>) {
        vec.push(b'l');
        for item in list {
            self.bencode_type(item, vec);
        }
        vec.push(b'e');
    }

    /// Encodes a dictionary with Bencode format.
    /// This function only works for a info value dictionary!!
    fn bencode_dictionary(&self, dic: &HashMap<String, BencodeType>, vec: &mut Vec<u8>) {
        vec.push(b'd');
        let sorted_keys = ["length", "name", "piece length", "pieces"];
        for key in sorted_keys {
            let value = dic.get(key);
            if let Some(v) = value {
                self.bencode_string(key.as_bytes(), vec);
                self.bencode_type(v, vec);
            }
        }
        vec.push(b'e');
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::parsers::bencode::BencodeParser;

    #[test]
    fn bencode_integer() {
        let integer_parsed = BencodeType::Integer(12345);
        let integer_bencoded = Encoder.bencode(&integer_parsed);

        let expected_vec = "i12345e".as_bytes().to_vec();
        assert_eq!(integer_bencoded, expected_vec);
    }

    #[test]
    fn bencode_string() {
        let string_parsed = BencodeType::String("hello bittorrent".as_bytes().to_vec());
        let string_bencoded = Encoder.bencode(&string_parsed);

        let expected_vec = "16:hello bittorrent".as_bytes().to_vec();
        assert_eq!(string_bencoded, expected_vec);
    }

    #[test]
    fn bencode_list() {
        let list_parsed = BencodeType::List(vec![
            BencodeType::Integer(1),
            BencodeType::Integer(2),
            BencodeType::Integer(3),
        ]);
        let list_bencoded = Encoder.bencode(&list_parsed);

        let expected_vec = "li1ei2ei3ee".as_bytes().to_vec();
        assert_eq!(list_bencoded, expected_vec);
    }

    #[test]
    fn bencode_info_dic() {
        let file_parsed = BencodeParser.parse_file("bencoded_files_testing/dictionary.txt");
        if let Ok(BencodeType::Dictionary(d)) = file_parsed {
            let info_value = d.get("info");
            if let Some(v) = info_value {
                let bencoded_info = Encoder.bencode(&v);
                let expected_vec= "d6:lengthi3379068928e4:name32:ubuntu-20.04.4-desktop-amd64.iso12:piece lengthi262144e6:pieces3:xyze".as_bytes().to_vec();
                assert_eq!(bencoded_info, expected_vec);
            } else {
                assert!(false);
            }
        } else {
            assert!(false);
        }
    }

    #[test]
    fn urlencode_case_1() {
        let decoded_data = "hola0129._-~";
        let encoded_data = Encoder.urlencode(&decoded_data.as_bytes());

        assert_eq!(encoded_data, decoded_data);
    }

    #[test]
    fn urlencode_case_2() {
        let decoded_data = "&#hola0129._-~:;";
        let encoded_data = Encoder.urlencode(&decoded_data.as_bytes());
        let expected_value = "%26%23hola0129._-~%3A%3B";
        assert_eq!(encoded_data, expected_value);
    }
}
