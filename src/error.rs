use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io;

#[derive(Debug)]
pub enum MessageParseError {
    UnknownOpcode(u8),
    UnexpectedEnd,
    InvalidChecksum,
    Update
}

impl Display for MessageParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::UnknownOpcode(opc) => write!(f, "unknown opcode: {:x}", opc),
            Self::UnexpectedEnd => write!(f, "unexpected end of stream"),
            Self::InvalidChecksum => write!(f, "invalid checksum"),
            Self::Update => write!(f, "update")
        }
    }
}

impl Error for MessageParseError {}

impl From<io::Error> for MessageParseError {
    fn from(_: io::Error) -> Self {
        todo!()
    }
}
