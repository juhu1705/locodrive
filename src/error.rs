use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum MessageParseError {
    UnknownOpcode(u8),
    UnexpectedEnd,
    InvalidChecksum,
}

impl Display for MessageParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::UnknownOpcode(opc) => write!(f, "unknown opcode: {:x}", opc),
            Self::UnexpectedEnd => write!(f, "unexpected end of stream"),
            Self::InvalidChecksum => write!(f, "invalid checksum"),
        }
    }
}

impl Error for MessageParseError {}
