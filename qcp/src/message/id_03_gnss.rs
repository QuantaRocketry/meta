use super::MessageTrait;

use defmt::*;
use zerocopy::*;

#[derive(
    Debug, Eq, PartialEq, Copy, Clone, Format, FromBytes, IntoBytes, KnownLayout, Immutable,
)]
#[repr(C)]
pub struct GNSS {
    pub latitude: u32,
    pub longitude: u32,
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use crate::PACKET_SIZE_MAX;

    #[test]
    fn direct() {
        let mut buf: [u8; PACKET_SIZE_MAX] = [0; PACKET_SIZE_MAX];

        GNSS {
            latitude: 132,
            longitude: 2,
        }
        .write_to(&mut buf[..size_of::<GNSS>()])
        .unwrap();

        std::assert_eq!(&[132, 0, 0, 0, 2, 0, 0, 0], &buf[..size_of::<GNSS>()]);

        let out = GNSS::ref_from_bytes(&buf[..size_of::<GNSS>()]).unwrap();
        std::assert_eq!(
            out,
            &GNSS {
                latitude: 132,
                longitude: 2
            }
        );
    }
}
