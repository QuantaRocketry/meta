pub mod snazzy {
    pub mod items {
        include!(concat!(env!("OUT_DIR"), "/snazzy.items.rs"));
    }
}

pub mod error;
pub mod header;
pub mod packet;

pub const PACKET_SIZE_MAX: usize = 32;

const _: () = assert!(
    header::HEADER_SIZE + packet::MESSAGE_SIZE_MAX <= PACKET_SIZE_MAX,
    "Max packet size is invalid"
);
