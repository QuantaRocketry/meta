use serde::{Deserialize, Serialize};

pub mod protocol;
pub use protocol::Protocol;

/// LoRa spreading factor (chips/symbol). Higher = more range, slower + more
/// airtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpreadingFactor {
    SF5,
    SF6,
    SF7,
    SF8,
    SF9,
    SF10,
    SF11,
    SF12,
}

/// LoRa signal bandwidth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoraBandwidth {
    Bw7,
    Bw10,
    Bw15,
    Bw20,
    Bw31,
    Bw41,
    Bw62,
    Bw125,
    Bw250,
    Bw500,
}

/// LoRa forward error correction coding rate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CodingRate {
    Cr45,
    Cr46,
    Cr47,
    Cr48,
}

/// LoRa-specific radio settings.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LoraConfig {
    pub spreading_factor: SpreadingFactor,
    pub bandwidth: LoraBandwidth,
    pub coding_rate: CodingRate,
    /// Enable for symbol durations >= 16.38ms (high SF + narrow BW).
    pub low_data_rate_opt: bool,
    /// Preamble length in symbols.
    pub preamble_length: u16,
    /// Append/check a CRC on the payload.
    pub crc_enabled: bool,
    /// Invert I/Q — set true on one side of a link if you want to avoid
    /// hearing your own retransmissions (common convention: gateways invert,
    /// nodes don't).
    pub invert_iq: bool,
}

/// GFSK pulse shaping filter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GfskPulseShape {
    NoFilter,
    Bt03,
    Bt05,
    Bt07,
    Bt1,
}

/// GFSK RX channel filter bandwidth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GfskBandwidth {
    Bw48,
    Bw58,
    Bw73,
    Bw97,
    Bw117,
    Bw146,
    Bw195,
    Bw234,
    Bw293,
    Bw39,
    Bw469,
    Bw586,
    Bw782,
    Bw938,
    Bw1173,
    Bw1562,
    Bw1872,
    Bw2323,
    Bw3120,
    Bw3736,
    Bw4670,
}

/// GFSK-specific radio settings.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GfskConfig {
    /// Bit rate in bits/second.
    pub bit_rate: u32,
    /// Frequency deviation in Hz.
    pub freq_deviation: u32,
    pub pulse_shape: GfskPulseShape,
    pub bandwidth: GfskBandwidth,
    /// Preamble length in bytes.
    pub preamble_length: u16,
    /// Sync word length in bytes.
    pub sync_word_length: u8,
}

/// The modulation scheme in use, along with its scheme-specific settings.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PhyConfig {
    Lora(LoraConfig),
    Gfsk(GfskConfig),
}

impl Default for LoraConfig {
    fn default() -> Self {
        Self {
            spreading_factor: SpreadingFactor::SF7,
            bandwidth: LoraBandwidth::Bw125,
            coding_rate: CodingRate::Cr45,
            low_data_rate_opt: false,
            preamble_length: 8,
            crc_enabled: true,
            invert_iq: false,
        }
    }
}

/// Radio configuration: carrier frequency plus modulation-specific settings.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RadioConfig {
    /// RF carrier frequency in Hz (e.g. `915_735_000` for 915.735 MHz).
    pub frequency_hz: u32,
    /// TX output power in dBm.
    pub tx_power_dbm: i8,
    pub phy: PhyConfig,
}

impl Default for RadioConfig {
    fn default() -> Self {
        Self {
            frequency_hz: 915_000_000,
            tx_power_dbm: 10,
            phy: PhyConfig::Lora(LoraConfig::default()),
        }
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
    use crate::assert_round_trip;

    #[test]
    fn radio_config_lora_round_trip() {
        assert_round_trip(
            &RadioConfig {
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
            .into(),
        );
    }

    #[test]
    fn radio_config_gfsk_round_trip() {
        assert_round_trip(
            &RadioConfig {
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
            .into(),
        );
    }
}
