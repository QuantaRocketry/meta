use crate::{error::Error, message::Message, message::MessageType};
mod version;
pub use version::ProtocolVersion;

use core::mem::size_of;
use zerocopy::*;

pub const HEADER_SIZE: usize = 6;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Immutable, KnownLayout, IntoBytes)]
#[repr(C, packed(1))]
pub struct Header {
    pub lrc: u8,
    pub version: ProtocolVersion,
    pub length: u8,
    pub message_type: MessageType,
    pub crc16: u16,
}

const _: () = assert!(
    HEADER_SIZE == size_of::<Header>(),
    "Somehow header is incorrect size"
);

impl Header {
    pub fn try_from_bytes(buf: &[u8]) -> Result<Self, Error> {
        if buf.len() != size_of::<Header>() {
            return Err(Error::BufferTooSmall);
        }

        let lrc = buf[0];

        let version = ProtocolVersion::try_ref_from_bytes(&buf[1..2])
            .map_err(|_| Error::UnknownVersion(buf[1]))?
            .clone();

        let length = buf[2];

        let message_type = MessageType::try_ref_from_bytes(&buf[3..4])
            .map_err(|_| Error::UnknownMessageType(buf[3]))?
            .clone();

        let crc16 = u16::ref_from_bytes(&buf[4..])
            .map_err(|_| Error::InvalidCRC16)?
            .clone();

        Ok(Header {
            lrc,
            version,
            length,
            message_type,
            crc16,
        })
    }
}

impl From<&Message> for Header {
    fn from(msg: &Message) -> Self {
        let todo = true; // TODO calculate all the checks
        Header {
            lrc: 0,
            version: ProtocolVersion::CURRENT_VERSION,
            length: 0,
            message_type: MessageType::from(msg),
            crc16: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use crate::PACKET_SIZE_MAX;

    #[test]
    fn direct() {
        let mut buf: [u8; PACKET_SIZE_MAX] = [0; PACKET_SIZE_MAX];

        std::println!("{:?}", size_of::<Header>());

        let header = Header {
            lrc: 0,
            version: ProtocolVersion::V1,
            length: 1,
            message_type: MessageType::Heartbeat,
            crc16: 0,
        };

        header.write_to(&mut buf[..size_of::<Header>()]).unwrap();
    }
}
