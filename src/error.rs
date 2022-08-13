use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io;

/// Represents an Error occurring when a message was received
/// but could not be passed correctly to a valid and known `LocoNet` message.
#[derive(Debug, Clone)]
pub enum MessageParseError {
    /// The OpCode of the message was unknown, maybe that code is not implemented yet.
    /// Please report this to the contributor.
    UnknownOpcode(u8),
    /// The messages length did not match the expected message length.
    UnexpectedEnd,
    /// Some expected message format bytes did not contain the expected value.
    InvalidFormat(String),
    /// The checksum could not be validated. The received message is corrupted. Please retry sending.
    InvalidChecksum,
    /// This is used only by the controller to receive and handle a shutdown request.
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
        MessageParseError::InvalidFormat(err.to_string())
    }
}

/// This error type is used to describe errors appearing on [`LocoNetConnector::send_message()`]
#[derive(Debug, Copy, Clone)]
pub enum LocoNetSendingError {
    /// If the reader is closed. This should not happen normally.
    /// If it happens your [`LocoNetConnector`] is corrupted and can no longer be used.
    IllegalState,
    /// The `LocoNet` does not respond in the specified time.
    Timeout,
    /// The `LocoNet` connection returns writing with an error.
    /// Please recheck your connection.
    NotWritable
}

impl Display for LocoNetSendingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Timeout => write!(f, "connection timed out"),
            Self::NotWritable => write!(f, "could not write to port"),
            Self::IllegalState => write!(f, "loco net connection in illegal state"),
        }
    }
}

impl Error for LocoNetSendingError {}
