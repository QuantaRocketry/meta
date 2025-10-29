#[cfg(test)]
mod tests {
    extern crate std;
    use prost::Message;
    use qcp::{
        PACKET_SIZE_MAX,
        header::HEADER_SIZE,
        packet::{self, Packet, PacketID},
    };

    macro_rules! packet_test {
        ($packet:ident, $data:expr, $expect:expr) => {
            paste::item! {
                #[test]
                fn [< direct_ $packet:lower >] () {
                    let mut buf = std::vec::Vec::with_capacity(PACKET_SIZE_MAX);
                    let p = $data;
                    p.encode(&mut buf).unwrap();

                    std::assert_eq!($expect, &buf[..]);
                    std::assert_eq!($expect.len(), p.encoded_len());

                    let out = packet::$packet::decode(&buf[..]);
                    std::assert_eq!(out, Ok(p));
                }

                #[test]
                fn [< indirect_ $packet:lower >] () {
                    let mut buf = std::vec::Vec::with_capacity(PACKET_SIZE_MAX);
                    let data = $data;
                    let p = Packet::from(data);
                    p.encode(&mut buf).unwrap();

                    std::assert_eq!($expect, &buf[HEADER_SIZE..]);
                    std::assert_eq!(HEADER_SIZE + $expect.len(), p.encoded_len());

                    let out = Packet::decode(&buf[..]).unwrap();
                    std::assert_eq!(
                        out,
                        Packet::$packet($data)
                    );
                }
            }
        };
    }

    packet_test!(
        Heartbeat,
        packet::Heartbeat { uptime: 13298326 },
        &[0x08, 0x96, 0xd5, 0xab, 0x06]
    );

    packet_test!(
        Request,
        packet::Request {
            packet_ids: vec![PacketID::Gnss as u32]
        },
        &[0x0A, 0x01, 0x03]
    );

    packet_test!(
        Gnss,
        packet::Gnss {
            latitude: 0.1f32,
            longitude: 0.1f32,
            altitude: 10.0f32,
        },
        &[
            0x0D, 0xCD, 0xCC, 0xCC, 0x3D, 0x15, 0xCD, 0xCC, 0xCC, 0x3D, 0x1D, 0x00, 0x00, 0x20,
            0x41
        ]
    );
}
