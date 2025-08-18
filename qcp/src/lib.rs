#![no_std]

pub mod error;
mod header;
pub mod message;

const PACKET_SIZE_MAX: usize = 32;

const _: () = assert!(
    header::HEADER_SIZE + message::MESSAGE_SIZE_MAX == PACKET_SIZE_MAX,
    "Max packet size is invalid"
);
