use crate::{
    error::Error,
    packet::{Packet, PacketID},
};
mod version;
use core::mem::size_of;
use prost::bytes::{Buf, BufMut};
pub use version::ProtocolVersion;

pub const HEADER_SIZE: usize = 6;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(C, packed(1))]
pub struct Header {
    pub version: ProtocolVersion,
    pub length: u16,
    pub message_type: PacketID,
    pub crc16: u16,
}

const _: () = assert!(
    HEADER_SIZE == size_of::<Header>(),
    "Somehow header is incorrect size"
);

impl Header {
    pub fn encode(&self, buf: &mut impl BufMut) -> Result<usize, Error> {
        let remaining = buf.remaining_mut();
        if remaining < size_of::<Self>() {
            return Err(Error::InvalidBufferSize);
        };

        buf.put_u8(self.version as u8);
        buf.put_u16(self.length);
        buf.put_u8(self.message_type as u8);
        buf.put_u16(self.crc16);

        Ok(1)
    }

    pub fn decode(mut buf: impl Buf) -> Result<Self, Error> {
        if buf.remaining() < size_of::<Self>() {
            return Err(Error::InvalidBufferSize);
        }

        let version: ProtocolVersion = buf.try_get_u8()?.try_into()?;
        let length = buf.try_get_u16()?;
        let message_type: PacketID = buf.try_get_u8()?.try_into()?;
        let crc16 = buf.try_get_u16()?;

        Ok(Header {
            version,
            length,
            message_type,
            crc16,
        })
    }
}

impl From<&Packet> for Header {
    fn from(p: &Packet) -> Self {
        let todo = true; // TODO calculate all the checks
        Header {
            version: ProtocolVersion::CURRENT_VERSION,
            length: 0,
            message_type: PacketID::from(p),
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
        let mut slice = &mut buf[..];

        std::println!("{:?}", size_of::<Header>());

        let header = Header {
            version: ProtocolVersion::V1,
            length: 1,
            message_type: PacketID::Heartbeat,
            crc16: 0,
        };

        header.encode(&mut slice).unwrap();
    }
}
