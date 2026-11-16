//! The LoRa waveform: what is on the air, not how a particular chip is asked to make it.

use serde::{Deserialize, Serialize};

/// Spreading factor: each symbol is `2^SF` chips. Higher is slower and more robust.
///
/// SF5 and SF6 are not available on every radio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
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

impl SpreadingFactor {
    /// The spreading factor as a number, 5 to 12.
    pub const fn value(self) -> u8 {
        match self {
            SpreadingFactor::SF5 => 5,
            SpreadingFactor::SF6 => 6,
            SpreadingFactor::SF7 => 7,
            SpreadingFactor::SF8 => 8,
            SpreadingFactor::SF9 => 9,
            SpreadingFactor::SF10 => 10,
            SpreadingFactor::SF11 => 11,
            SpreadingFactor::SF12 => 12,
        }
    }
}

/// Chirp bandwidth. The names are the nominal kHz (`Bw7` is 7.8 kHz, `Bw31` is 31.25 kHz).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
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

impl LoraBandwidth {
    /// The bandwidth in Hz (fractions truncated: `Bw7` is 7812.5 Hz).
    pub const fn hz(self) -> u32 {
        match self {
            LoraBandwidth::Bw7 => 7_812,
            LoraBandwidth::Bw10 => 10_416,
            LoraBandwidth::Bw15 => 15_625,
            LoraBandwidth::Bw20 => 20_833,
            LoraBandwidth::Bw31 => 31_250,
            LoraBandwidth::Bw41 => 41_666,
            LoraBandwidth::Bw62 => 62_500,
            LoraBandwidth::Bw125 => 125_000,
            LoraBandwidth::Bw250 => 250_000,
            LoraBandwidth::Bw500 => 500_000,
        }
    }
}

/// Forward error correction rate: 4 data bits are sent as 5 to 8 coded bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum CodingRate {
    Cr45,
    Cr46,
    Cr47,
    Cr48,
}

/// Whether the packet carries a PHY header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum LoraHeader {
    /// The header announces the payload length, coding rate and whether a CRC follows.
    Explicit,
    /// No header: both ends must already agree on the length, coding rate and CRC.
    Implicit { payload_length: u8 },
}

/// The LoRa waveform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct LoraConfig {
    pub spreading_factor: SpreadingFactor,
    pub bandwidth: LoraBandwidth,
    pub coding_rate: CodingRate,
    /// Preamble length in symbols (4.25 more are sent on air).
    pub preamble_length: u16,
    pub header: LoraHeader,
    /// Whether a CRC is appended to (and checked on) the payload.
    pub crc_enabled: bool,
    /// Whether the I and Q signals are inverted, which flips the chirp direction.
    /// A common convention is for gateways to invert and nodes not to, so nodes
    /// do not hear each other's uplinks.
    pub invert_iq: bool,
    /// The sync word that separates networks: `0x12` for private networks and
    /// `0x34` for LoRaWAN. Radios store it differently; this is the 8-bit form.
    pub sync_word: u8,
}

impl Default for LoraConfig {
    fn default() -> Self {
        Self {
            spreading_factor: SpreadingFactor::SF7,
            bandwidth: LoraBandwidth::Bw125,
            coding_rate: CodingRate::Cr45,
            preamble_length: 8,
            header: LoraHeader::Explicit,
            crc_enabled: true,
            invert_iq: false,
            sync_word: 0x12,
        }
    }
}

impl LoraConfig {
    /// Duration of one symbol in microseconds: `2^SF / bandwidth`.
    pub const fn symbol_time_us(&self) -> u32 {
        ((1u64 << self.spreading_factor.value()) * 1_000_000 / self.bandwidth.hz() as u64) as u32
    }

    /// Whether the low data rate optimisation is in use. It changes how bits are
    /// spread over symbols, so both ends must agree; the LoRa rule is that it is
    /// on when a symbol lasts 16 ms or more, so it follows from the other
    /// settings instead of being one of them.
    pub const fn low_data_rate_optimize(&self) -> bool {
        self.symbol_time_us() >= 16_000
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symbol_time() {
        // 2^7 / 125 kHz = 1.024 ms
        assert_eq!(LoraConfig::default().symbol_time_us(), 1_024);
    }

    #[test]
    fn low_data_rate_optimize_follows_the_16_ms_rule() {
        let cfg = |sf, bandwidth| LoraConfig { spreading_factor: sf, bandwidth, ..Default::default() };
        // Exactly 16.384 ms: on.
        assert!(cfg(SpreadingFactor::SF11, LoraBandwidth::Bw125).low_data_rate_optimize());
        assert!(cfg(SpreadingFactor::SF12, LoraBandwidth::Bw250).low_data_rate_optimize());
        assert!(cfg(SpreadingFactor::SF7, LoraBandwidth::Bw7).low_data_rate_optimize());
        // Half of that: off.
        assert!(!cfg(SpreadingFactor::SF10, LoraBandwidth::Bw125).low_data_rate_optimize());
        assert!(!cfg(SpreadingFactor::SF12, LoraBandwidth::Bw500).low_data_rate_optimize());
        assert!(!LoraConfig::default().low_data_rate_optimize());
    }
}
