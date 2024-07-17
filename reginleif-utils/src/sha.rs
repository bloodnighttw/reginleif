//! This module contains the SHA struct and the TryFrom implementation for SHA.
//! You can use this struct to store SHA1 and SHA256,
//! and you can convert &str to SHA by using TryFrom trait.

use std::num::ParseIntError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// SHA enum
#[derive(Debug,Clone,PartialEq)]
pub enum SHA{
    SHA1(Vec<u8>),
    SHA256(Vec<u8>)
}

impl Serialize for SHA{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        let hex:String = self.into();
        serializer.serialize_str(hex.as_str())
    }
}

impl<'de> Deserialize<'de> for SHA{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        let string = String::deserialize(deserializer)?;
        let sha = string.try_into().map_err(serde::de::Error::custom)?;
        Ok(sha)
    }
}

/// decode hex to u8 array
fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

/// decode hex to array
impl TryFrom<&str> for SHA{
    type Error = ParseIntError;

    /// decode str to SHA
    fn try_from(value: &str) -> Result<Self, Self::Error> {

        let decode = decode_hex(value)?;

        if decode.len() == 20 {
            let decode = decode_hex(value)?;
            return Ok(SHA::SHA1(decode))
        } else if decode.len() == 32{
            return Ok(SHA::SHA256(decode))
        }

        unreachable!("This should never be reached, checked your sha");
    }
}

impl TryFrom<String> for SHA{
    type Error = ParseIntError;

    /// convert String to SHA
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl From<&SHA> for String{
    /// convert SHA to String
    fn from(value: &SHA) -> Self {
        match value {
            SHA::SHA1(v) => {
                v.iter().map(|x| format!("{:2x}",x)).collect()
            }
            SHA::SHA256(v) => {
                v.iter().map(|x| format!("{:2x}",x)).collect()
            }
        }
    }
}

impl From<SHA> for String{
    /// encode SHA to String
    fn from(value: SHA) -> Self {
        Self::from(&value)
    }
}
