use std::collections::HashMap;
use std::io::{Error, ErrorKind};
/// # enum Bencode Type
/// Represents the four different data types supported by the Bencode format.
/// Also, one more type is End, which indicates the end of a structure.
#[derive(Debug, PartialEq, Clone)]
pub enum BencodeType {
    String(Vec<u8>),
    Integer(i64),
    List(Vec<BencodeType>),
    Dictionary(HashMap<String, BencodeType>),
    End,
}

impl BencodeType {
    pub fn get_value_from_dict(&self, key: &str) -> Result<BencodeType, Error> {
        if let BencodeType::Dictionary(dict) = self {
            if let Some(value) = dict.get(key) {
                return Ok(value.clone());
            }
        }
        Err(Error::new(
            ErrorKind::InvalidData,
            "It's not a dict or it has not received key",
        ))
    }

    pub fn get_string(&self) -> Result<Vec<u8>, Error> {
        if let BencodeType::String(s) = self {
            return Ok(s.to_vec());
        }
        Err(Error::new(ErrorKind::InvalidData, "It's not a string"))
    }

    pub fn get_integer(&self) -> Result<i64, Error> {
        if let BencodeType::Integer(i) = self {
            return Ok(*i);
        }
        Err(Error::new(ErrorKind::InvalidData, "It's not a integer"))
    }

    pub fn get_list(&self) -> Result<Vec<BencodeType>, Error> {
        if let BencodeType::List(l) = self {
            return Ok(l.clone());
        }
        Err(Error::new(ErrorKind::InvalidData, "It's not a list"))
    }
}
