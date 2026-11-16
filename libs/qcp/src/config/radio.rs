use serde::{Deserialize, Serialize};

pub mod gfsk;
pub mod lora;
pub mod protocol;

pub use gfsk::{
    DataCoding, GfskConfig, GfskCrc, GfskPulseShape, PacketLength, PreamblePattern, SyncWord,
};
pub use lora::{CodingRate, LoraBandwidth, LoraConfig, LoraHeader, SpreadingFactor};
pub use protocol::Protocol;

/// The modulation scheme in use, along with its scheme-specific waveform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum PhyConfig {
    Lora(LoraConfig),
    Gfsk(GfskConfig),
}

/// The waveform on the air: the carrier frequency plus the modulation.
///
/// This describes what is transmitted, independent of any particular radio.
/// Each app maps it onto the settings of the radio it has, and chooses the
/// radio-specific ones itself (transmit power, receiver filter bandwidth, ...).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct RadioConfig {
    /// Carrier frequency in Hz (e.g. `915_735_000` for 915.735 MHz).
    pub frequency_hz: u32,
    pub phy: PhyConfig,
}

impl Default for RadioConfig {
    fn default() -> Self {
        Self { frequency_hz: 915_000_000, phy: PhyConfig::Lora(LoraConfig::default()) }
    }
}

/// The user-facing radio selection. This is what gets stored and edited;
/// the radio itself is driven by the [`RadioConfig`] it resolves to.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RadioSettings {
    pub protocol: Protocol,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MESSAGE_SIZE_MAX, Message, assert_round_trip, encode};

    fn eggtimer_like() -> GfskConfig {
        GfskConfig {
            bit_rate: 9600,
            freq_deviation: 35_000,
            pulse_shape: GfskPulseShape::Bt05,
            preamble_pattern: PreamblePattern::Alternating01,
            preamble_length: 5,
            sync_word: SyncWord::new(&[0x2D, 0xD4]).unwrap(),
            length: PacketLength::Variable,
            crc: GfskCrc::xmodem(true),
            coding: DataCoding::None,
        }
    }

    #[test]
    fn radio_config_lora_round_trip() {
        assert_round_trip(
            &RadioConfig {
                frequency_hz: 915_735_000,
                phy: PhyConfig::Lora(LoraConfig {
                    spreading_factor: SpreadingFactor::SF9,
                    bandwidth: LoraBandwidth::Bw250,
                    coding_rate: CodingRate::Cr47,
                    preamble_length: 12,
                    header: LoraHeader::Implicit { payload_length: 20 },
                    crc_enabled: false,
                    invert_iq: true,
                    sync_word: 0x34,
                }),
            }
            .into(),
        );
    }

    #[test]
    fn radio_config_gfsk_round_trip() {
        assert_round_trip(
            &RadioConfig { frequency_hz: 433_000_000, phy: PhyConfig::Gfsk(eggtimer_like()) }
                .into(),
        );
    }

    #[test]
    fn default_radio_config_round_trip() {
        assert_round_trip(&RadioConfig::default().into());
    }

    #[test]
    fn largest_radio_config_fits_in_a_message() {
        let big = |phy| RadioConfig { frequency_hz: u32::MAX, phy };
        let gfsk = GfskConfig {
            bit_rate: u32::MAX,
            freq_deviation: u32::MAX,
            preamble_length: u16::MAX,
            sync_word: SyncWord::new(&[0xFF; 8]).unwrap(),
            length: PacketLength::Fixed(u8::MAX),
            crc: GfskCrc::Crc16 {
                polynomial: u16::MAX,
                initial: u16::MAX,
                invert_output: true,
                include_length: true,
            },
            coding: DataCoding::Whitening,
            ..eggtimer_like()
        };
        let lora = LoraConfig {
            preamble_length: u16::MAX,
            header: LoraHeader::Implicit { payload_length: u8::MAX },
            ..LoraConfig::default()
        };
        for phy in [PhyConfig::Gfsk(gfsk), PhyConfig::Lora(lora)] {
            let mut buf = [0u8; MESSAGE_SIZE_MAX];
            assert!(encode(&Message::RadioConfig(big(phy)), &mut buf).is_ok());
        }
    }
}
