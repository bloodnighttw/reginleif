use std::array::TryFromSliceError;
use std::num::ParseIntError;
use thiserror::Error;

#[derive(Debug)]
pub enum SHA{
    SHA1([u8;20]),
    SHA256([u8;32])
}

#[derive(Debug,Error)]
pub enum TryFromSHAHexError{
    #[error("ParseIntError: {0}")]
    ParseIntError(ParseIntError),
    #[error("TryFromSliceError: {0}")]
    TryFromSliceError(TryFromSliceError)
}

impl From<TryFromSliceError> for TryFromSHAHexError{
    fn from(value: TryFromSliceError) -> Self {
        TryFromSHAHexError::TryFromSliceError(value)
    }
}

impl From<ParseIntError> for TryFromSHAHexError{
    fn from(value: ParseIntError) -> Self {
        TryFromSHAHexError::ParseIntError(value)
    }
}

fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

/// decode hex to array
impl TryFrom<&str> for SHA{
    type Error = TryFromSHAHexError;


    fn try_from(value: &str) -> Result<Self, Self::Error> {

        if value.len() == 40 {
            let decode = decode_hex(value)?;
            let convert = decode.iter().as_slice().try_into()?;
            return Ok(SHA::SHA1(convert))
        } else if value.len() == 64{
            let decode = decode_hex(value)?;
            let convert = decode.iter().as_slice().try_into()?;
            return Ok(SHA::SHA256(convert))
        }

        unreachable!("This should never be reached, checked your sha");
    }
}

impl From<SHA> for String{
    fn from(value: SHA) -> Self {
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

mod test{
    use crate::sha::SHA;

    #[test]
    fn test() -> anyhow::Result<()>{
        let sha:SHA = "8ab31282892976da4695f5d721567f9584e1c6e69e9fef637b73f8cdc7adbcef".try_into()?;
        let sha_string2:String = sha.into();
        
        assert_eq!("8ab31282892976da4695f5d721567f9584e1c6e69e9fef637b73f8cdc7adbcef",sha_string2);
        
        Ok(())
        
    }
}