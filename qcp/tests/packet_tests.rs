#[cfg(test)]
mod tests {
    extern crate std;
    use prost::Message;
    use qcp::{
        PACKET_SIZE_MAX,
        packet::{self, Packet},
        header,
    };

    #[test]
    fn direct() {
        let mut buf = [0u8; PACKET_SIZE_MAX];
        let p = packet::Heartbeat { uptime: 13298326 };
        {
            let mut slice = &mut buf[..];
            p.encode(&mut slice).unwrap();
        }

        std::assert_eq!(5, p.encoded_len());
        let encoded_data = &buf[..p.encoded_len()];
        std::assert_eq!(&[0x08, 0x96, 0xd5, 0xab, 0x06], encoded_data);

        std::println!("{:?}", buf);
        // FIXME Why doesn't this work?
        // let out = packet::Heartbeat::decode(&buf[..]);
        let out = packet::Heartbeat::decode(&buf[..p.encoded_len()]);
        std::assert_eq!(out, Ok(p));
    }

    #[test]
    fn indirect() {
        let mut buf = std::vec::Vec::with_capacity(PACKET_SIZE_MAX);
        Packet::Heartbeat(packet::Heartbeat { uptime: 13298326 })
            .encode(&mut buf)
            .unwrap();

        std::assert_eq!(&[0x08, 0x96, 0xd5, 0xab, 0x06], &buf[header::HEADER_SIZE..]);

        std::println!("{:?}", &buf);

        let out = Packet::decode(&*buf).unwrap();
        std::assert_eq!(out, Packet::Heartbeat(packet::Heartbeat { uptime: 13298326 }));
    }
}
