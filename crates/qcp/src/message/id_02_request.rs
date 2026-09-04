use super::MessageTrait;

use defmt::*;
use zerocopy::*;

#[derive(
    Debug, Eq, PartialEq, Copy, Clone, Format, FromBytes, IntoBytes, KnownLayout, Immutable,
)]
#[repr(C)]
pub struct Request {
    pub messages: u32,
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use crate::PACKET_SIZE_MAX;

    #[test]
    fn direct() {
        let mut buf: [u8; PACKET_SIZE_MAX] = [0; PACKET_SIZE_MAX];
        Request { messages: 13298326 }
            .write_to(&mut buf[..size_of::<Request>()])
            .unwrap();

        std::assert_eq!(&[150, 234, 202, 0], &buf[..size_of::<Request>()]);

        let out: &Request = Request::ref_from_bytes(&buf[..size_of::<Request>()]).unwrap();
        std::assert_eq!(out, &Request { messages: 13298326 });
    }
}
