use zerocopy::*;

use crate::error::Error;

/// The identifying byte for the protocol.
#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromBytes, Immutable, IntoBytes, KnownLayout)]
#[repr(u8)]
pub enum ProtocolVersion {
    V1 = 0x01,
}

impl ProtocolVersion {
    pub const CURRENT_VERSION: ProtocolVersion = ProtocolVersion::V1;
}

impl From<ProtocolVersion> for u8 {
    fn from(id: ProtocolVersion) -> u8 {
        id as u8
    }
}

impl TryFrom<u8> for ProtocolVersion {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(ProtocolVersion::V1),
            _ => Err(Error::UnknownVersion(value)),
        }
    }
}
