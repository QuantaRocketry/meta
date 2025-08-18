use super::MessageTrait;

use defmt::*;
use zerocopy::*;

#[derive(
    Debug, Eq, PartialEq, Copy, Clone, Format, FromBytes, IntoBytes, KnownLayout, Immutable,
)]
#[repr(C)]
pub struct Heartbeat {
    pub uptime: u32,
}

impl MessageTrait for Heartbeat {}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use crate::header::HEADER_SIZE;
    use crate::message::Message;
    use crate::PACKET_SIZE_MAX;

    #[test]
    fn direct() {
        let mut buf: [u8; PACKET_SIZE_MAX] = [0; PACKET_SIZE_MAX];
        Heartbeat { uptime: 13298326 }
            .write_to(&mut buf[..size_of::<Heartbeat>()])
            .unwrap();

        std::assert_eq!(&[150, 234, 202, 0], &buf[..size_of::<Heartbeat>()]);

        let out: &Heartbeat = Heartbeat::ref_from_bytes(&buf[..size_of::<Heartbeat>()]).unwrap();
        std::assert_eq!(out, &Heartbeat { uptime: 13298326 });
    }

    #[test]
    fn indirect() {
        let mut buf: [u8; PACKET_SIZE_MAX] = [0; PACKET_SIZE_MAX];

        Message::Heartbeat(Heartbeat { uptime: 13298326 })
            .encode_to(&mut buf)
            .unwrap();

        std::assert_eq!(&[150, 234, 202, 0], &buf[HEADER_SIZE..HEADER_SIZE + 4]);

        let out = Message::try_decode_from_bytes(&buf).unwrap();

        std::assert_eq!(out, Message::Heartbeat(Heartbeat { uptime: 13298326 }));
    }
}
