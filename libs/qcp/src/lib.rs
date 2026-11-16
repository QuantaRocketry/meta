#![cfg_attr(not(test), no_std)]

pub mod config;
mod coordinate;
mod message;
mod state;

pub use config::radio::{
    CodingRate, GfskBandwidth, GfskConfig, GfskPulseShape, LoraBandwidth, LoraConfig, PhyConfig,
    Protocol, RadioConfig, RadioSettings, SpreadingFactor,
};
pub use coordinate::Coordinate;
pub use message::{Message, MessageKind};
pub use postcard::{Error, Result};
pub use state::{BatteryState, ChargeStatus, TrackedPOI};

/// Largest number of bytes a single encoded `Message` can occupy on the wire.
///
/// Sized for a `TrackedPOI` with a full-length `id`, plus headroom for
/// future message variants.
pub const MESSAGE_SIZE_MAX: usize = 64;

/// Serialize `message` into `buf`, returning the slice of `buf` that was
/// written to.
pub fn encode<'a>(message: &Message, buf: &'a mut [u8]) -> Result<&'a mut [u8]> {
    postcard::to_slice(message, buf)
}

/// Deserialize a `Message` from `buf`.
pub fn decode(buf: &[u8]) -> Result<Message> {
    postcard::from_bytes(buf)
}

/// Encode then decode `message`, asserting it survives the trip unchanged.
#[cfg(test)]
pub(crate) fn assert_round_trip(message: &Message) {
    let mut buf = [0u8; MESSAGE_SIZE_MAX];

    let encoded = encode(message, &mut buf).unwrap();
    let decoded = decode(encoded).unwrap();

    assert_eq!(&decoded, message);
}
