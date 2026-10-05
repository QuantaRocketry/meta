#![cfg_attr(not(test), no_std)]

pub mod config;
mod coordinate;
mod message;
mod state;

pub use config::{
    radio::{
        CodingRate, GfskBandwidth, GfskConfig, GfskPulseShape, LoraBandwidth, LoraConfig,
        PhyConfig, Protocol, RadioConfig, RadioSettings, SpreadingFactor,
    },
};
pub use coordinate::Coordinate;
pub use message::Message;
pub use postcard::{Error, Result};
pub use state::TrackedPOI;

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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_poi() -> TrackedPOI {
        let mut poi = TrackedPOI {
            coordinate: Coordinate {
                latitude: 33.123123,
                longitude: -151.456456,
                altitude: 12.5,
            },
            ..Default::default()
        };
        poi.id.push_str("VK2GTX").unwrap();
        poi
    }

    #[test]
    fn round_trip() {
        let message: Message = sample_poi().into();
        let mut buf = [0u8; MESSAGE_SIZE_MAX];

        let encoded = encode(&message, &mut buf).unwrap();
        let decoded = decode(encoded).unwrap();

        assert_eq!(decoded, message);
    }

    #[test]
    fn radio_config_lora_round_trip() {
        let message: Message = RadioConfig {
            frequency_hz: 915_735_000,
            tx_power_dbm: 10,
            phy: PhyConfig::Lora(LoraConfig {
                spreading_factor: SpreadingFactor::SF7,
                bandwidth: LoraBandwidth::Bw125,
                coding_rate: CodingRate::Cr45,
                low_data_rate_opt: false,
                preamble_length: 8,
                crc_enabled: true,
                invert_iq: false,
            }),
        }
        .into();
        let mut buf = [0u8; MESSAGE_SIZE_MAX];

        let encoded = encode(&message, &mut buf).unwrap();
        let decoded = decode(encoded).unwrap();

        assert_eq!(decoded, message);
    }

    #[test]
    fn radio_config_gfsk_round_trip() {
        let message: Message = RadioConfig {
            frequency_hz: 433_000_000,
            tx_power_dbm: 10,
            phy: PhyConfig::Gfsk(GfskConfig {
                bit_rate: 9600,
                freq_deviation: 5000,
                pulse_shape: GfskPulseShape::Bt05,
                bandwidth: GfskBandwidth::Bw117,
                preamble_length: 16,
                sync_word_length: 4,
            }),
        }
        .into();
        let mut buf = [0u8; MESSAGE_SIZE_MAX];

        let encoded = encode(&message, &mut buf).unwrap();
        let decoded = decode(encoded).unwrap();

        assert_eq!(decoded, message);
    }

    #[test]
    fn protocol_round_trip() {
        let message: Message = Protocol::Eggtimer.into();
        let mut buf = [0u8; MESSAGE_SIZE_MAX];

        let encoded = encode(&message, &mut buf).unwrap();
        let decoded = decode(encoded).unwrap();

        assert_eq!(decoded, message);
    }

    #[test]
    fn request_round_trip() {
        let message = Message::Request(7);
        let mut buf = [0u8; MESSAGE_SIZE_MAX];

        let encoded = encode(&message, &mut buf).unwrap();
        let decoded = decode(encoded).unwrap();

        assert_eq!(decoded, message);
    }

    #[test]
    fn fits_within_message_size_max() {
        let mut poi = sample_poi();
        // Worst case: a fully-populated, maximum-length id.
        poi.id.clear();
        poi.id.push_str("0123456789ABCDEF").unwrap();
        let message: Message = poi.into();

        let mut buf = [0u8; MESSAGE_SIZE_MAX];
        let encoded = encode(&message, &mut buf).unwrap();
        assert!(encoded.len() <= MESSAGE_SIZE_MAX);
    }
}
