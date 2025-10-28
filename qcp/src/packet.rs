use crate::{error, header};
use prost::Message;
use prost::bytes::{Buf, BufMut};

mod data {
    include!(concat!(env!("OUT_DIR"), "/packets.rs"));
}
pub use data::*;

macro_rules! decode_to_message_data {
    ($variant:ident, $buf:expr) => {
        Packet::$variant(data::$variant::decode($buf)?)
    };
}

macro_rules! define_packet_ids {
    ( $( ($variant:ident, $id:literal) ),+ $(,)? ) => {
        #[derive(Debug, PartialEq, Eq, Clone, Copy)]
        #[repr(u8)]
        pub enum PacketID {
            $(
                $variant = $id,
            )*
        }

        impl TryFrom<u8> for PacketID {
            type Error = error::Error;

            fn try_from(value: u8) -> Result<Self, error::Error> {
                Ok(match value {
                    $(
                        $id => PacketID::$variant,
                    )*
                    _ => {return Err(error::Error::UnknownVersion(value));}
                })
            }
        }

        #[derive(Debug, PartialEq, Clone)]
        pub enum Packet {
            $(
                $variant(data::$variant),
            )*
        }

        impl From<&Packet> for u8 {
            fn from(value: &Packet) -> Self {
                match value {
                    $(
                        Packet::$variant(_) => $id as u8,
                    )*
                }
            }
        }

        impl From<Packet> for u8 {
            fn from(value: Packet) -> Self {
                (&value).into()
            }
        }

        impl From<&Packet> for PacketID {
            fn from(value: &Packet) -> Self {
                match value {
                    $(
                        Packet::$variant(_) => PacketID::$variant,
                    )*
                }
            }
        }

        impl From<Packet> for PacketID {
            fn from(value: Packet) -> Self {
                (&value).into()
            }
        }

        $(
            impl From<data::$variant> for Packet {
                #[inline]
                fn from(value: data::$variant) -> Self {
                    Packet::$variant(value)
                }
            }
        )*

        impl TryFrom<&[u8]> for Packet {
            type Error = error::Error;

            fn try_from(buf: &[u8]) -> Result<Self, Self::Error> {
                let header = header::Header::decode(buf)?;
                let data = match header.message_type {
                    $(
                        PacketID::$variant => decode_to_message_data!($variant, buf),
                    )*
                };
                Ok(data)
            }
        }

        impl Packet {
            pub fn encode(&self, buf: &mut impl BufMut) -> Result<usize, error::Error> {
                let h = header::Header::from(self);
                h.encode(buf)?;
                let mut num_encoded = header::HEADER_SIZE;
                match self {
                    $(
                        Self::$variant(data) => {
                            data.encode(buf)?;
                            num_encoded += data.encoded_len();
                        },
                    )*
                };
                Ok(num_encoded)
            }

            pub fn decode(mut buf: impl Buf) -> Result<Packet, error::Error> {
                let h = header::Header::decode(&mut buf)?;
                Ok(match h.message_type {
                    $(
                        PacketID::$variant => Packet::$variant(data::$variant::decode(buf).map_err(|_| error::Error::InvalidData)?),
                    )*
                })
            }
        }
    };
}

define_packet_ids! {
    (Heartbeat, 1),
    (Request, 2),
    (Gnss, 3),
}

pub const MESSAGE_SIZE_MAX: usize = core::mem::size_of::<Packet>();
const _: () = assert!(
    header::HEADER_SIZE + MESSAGE_SIZE_MAX <= crate::PACKET_SIZE_MAX,
    "Max packet size is invalid"
);
