use core::fmt;

use zerocopy::TryCastError;

/// Errors that can occur during protocol serialization or deserialization.
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// The provided buffer is the wrong size to hold the serialized message.
    InvalidBufferSize,
    /// The input bytes are too short to form a complete message.
    InputTooShort,
    /// An unknown `version` byte was encountered.
    UnknownVersion(u8),
    /// An unknown `MessageType` byte was encountered.
    UnknownMessageType(u8),
    /// The message payload length is invalid or exceeds limits.
    InvalidPayloadLength,
    /// Error converting bytes to a float (should not happen with correct data).
    InvalidFloatConversion,
    /// Error with CRC16.
    InvalidCRC16,
    /// The data is improperly aligned.
    InvalidAlignment,
    /// The data is invalid.
    InvalidData,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnknownVersion(byte) => {
                write!(f, "Unknown protocol version: {}", byte)
            }
            Error::UnknownMessageType(msg_type) => {
                write!(f, "Unknown message type: {}", msg_type)
            }
            Error::InvalidPayloadLength => {
                write!(f, "Invalid payload length specified in message")
            }
            Error::InvalidFloatConversion => write!(f, "Failed to convert bytes to float"),
            Error::InvalidCRC16 => write!(f, "Invalid CRC16"),
            Error::InvalidAlignment => write!(f, "Invalid alignment"),
            Error::InvalidBufferSize => todo!(),
            Error::InputTooShort => todo!(),
            Error::InvalidData => todo!(),
        }
    }
}

impl<T: zerocopy::TryFromBytes> From<zerocopy::ConvertError<T, T, T>> for Error {
    fn from(value: zerocopy::ConvertError<T, T, T>) -> Self {
        match value {
            zerocopy::ConvertError::Alignment(_) => Error::InvalidAlignment,
            zerocopy::ConvertError::Size(_) => Error::InvalidBufferSize,
            zerocopy::ConvertError::Validity(_) => Error::InvalidData,
        }
    }
}
