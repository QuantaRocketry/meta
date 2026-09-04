use core::fmt;
use prost::{EncodeError, bytes::TryGetError};

/// Errors that can occur during protocol serialization or deserialization.
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// The provided buffer is the wrong size to hold the serialized message.
    InvalidBufferSize,
    /// An unknown `version` byte was encountered.
    UnknownVersion(u8),
    /// An unknown `MessageType` byte was encountered.
    UnknownPacket(u8),
    /// Error with CRC16.
    InvalidCRC16,
    /// The data is invalid.
    InvalidData,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnknownVersion(byte) => {
                write!(f, "Unknown protocol version: {}", byte)
            }
            Error::UnknownPacket(packet) => {
                write!(f, "Unknown packet: {}", packet)
            }
            Error::InvalidCRC16 => write!(f, "Invalid CRC16"),
            Error::InvalidBufferSize => write!(f, "Invalid buffer size"),
            Error::InvalidData => write!(f, "Invalid data"),
        }
    }
}

impl From<TryGetError> for Error {
    fn from(_value: TryGetError) -> Self {
        Error::InvalidBufferSize
    }
}

impl From<prost::EncodeError> for Error {
    fn from(_value: EncodeError) -> Self {
        Error::InvalidBufferSize
    }
}

impl From<prost::DecodeError> for Error {
    fn from(_value: prost::DecodeError) -> Self {
        Error::InvalidData
    }
}
