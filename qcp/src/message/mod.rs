mod id_01_heartbeat;
pub use id_01_heartbeat::Heartbeat;

mod id_02_request;
pub use id_02_request::Request;

mod id_03_gnss;
pub use id_03_gnss::GNSS;

use crate::error::Error;
use crate::header::{Header, HEADER_SIZE};

use defmt::*;
use zerocopy::{CastError, ConvertError, Immutable, IntoBytes, KnownLayout, TryFromBytes};

pub const MESSAGE_SIZE_MAX: usize = crate::PACKET_SIZE_MAX - crate::header::HEADER_SIZE;

pub trait MessageTrait: TryFromBytes + Immutable + IntoBytes + KnownLayout {}

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromBytes, Immutable, IntoBytes, KnownLayout)]
#[repr(u8)]
pub enum MessageType {
    Heartbeat = 0x01,
    Request = 0x02,
    GNSS = 0x03,
}

impl TryFrom<u8> for MessageType {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(MessageType::try_ref_from_bytes(&[value])
            .map_err(|_| Error::UnknownMessageType(value))?
            .clone())
    }
}

impl From<MessageType> for u8 {
    fn from(value: MessageType) -> Self {
        value as u8
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Format)]
#[repr(u8)]
pub enum Message {
    Heartbeat(Heartbeat),
    Request(Request),
    GNSS(GNSS),
}

impl From<&Message> for MessageType {
    fn from(value: &Message) -> Self {
        match value {
            Message::Heartbeat(_) => MessageType::Heartbeat,
            Message::Request(_) => MessageType::Request,
            Message::GNSS(_) => MessageType::GNSS,
        }
    }
}

macro_rules! decode_to_message {
    ($variant:ident, $buf:expr) => {
        Message::$variant(
            $variant::try_ref_from_bytes(&$buf[..core::mem::size_of::<$variant>()])
                .map_err(|_| Error::InputTooShort)?
                .clone(),
        )
    };
}

impl Message {
    pub fn encode_to(self: &Self, buf: &mut [u8]) -> Result<(), Error> {
        if buf.len() < HEADER_SIZE {
            return Err(Error::InputTooShort);
        }

        let header = Header::from(self);
        header.write_to(&mut buf[..HEADER_SIZE]).map_err(|e| {
            let e: ConvertError<_, _, _> = e.into();
            let e: Error = Error::from(e);
        })?;

        let mut buf = &mut buf[HEADER_SIZE..];

        match self {
            Message::Heartbeat(msg) => msg.write_to(&mut buf[..size_of::<Heartbeat>()]).into(),
            Message::Request(msg) => msg.write_to(&mut buf[HEADER_SIZE..]).into(),
            Message::GNSS(msg) => msg.write_to(&mut buf).into(),
        }?;

        Ok(())
    }

    pub fn try_decode_from_bytes(buf: &[u8]) -> Result<Self, Error> {
        let header = Header::try_from_bytes(&buf[..HEADER_SIZE])?;

        let buf = &buf[HEADER_SIZE..];
        Ok(match header.message_type {
            MessageType::Heartbeat => decode_to_message!(Heartbeat, &buf),
            MessageType::Request => decode_to_message!(Request, &buf),
            MessageType::GNSS => decode_to_message!(GNSS, &buf),
        })
    }
}
