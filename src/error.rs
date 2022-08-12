use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io;

#[derive(Debug)]
pub enum MessageParseError {
    UnknownOpcode(u8),
    UnexpectedEnd,
    InvalidFormat(String),
    InvalidChecksum,
    Update,
}

impl Display for MessageParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::UnknownOpcode(opc) => write!(f, "unknown opcode: {:x}", opc),
            Self::UnexpectedEnd => write!(f, "unexpected end of stream"),
            Self::InvalidChecksum => write!(f, "invalid checksum"),
            Self::Update => write!(f, "update"),
            Self::InvalidFormat(ref message) => write!(f, "invalid format: {:?}", message)
        }
    }
}

impl Error for MessageParseError {}

impl From<io::Error> for MessageParseError {
    fn from(err: io::Error) -> Self {
        MessageParseError::InvalidFormat(err.to_string().into())
    }
}
