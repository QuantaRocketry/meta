//! The (G)FSK waveform: what is on the air, not how a particular chip is asked to make it.
//!
//! Receiver details such as the channel filter bandwidth are deliberately not
//! here; a radio picks them from the bit rate and deviation.

use serde::{Deserialize, Serialize};

/// Gaussian filter applied to the data before modulation, given as its
/// bandwidth-time product.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GfskPulseShape {
    /// Unfiltered: plain 2-FSK.
    NoFilter,
    Bt03,
    Bt05,
    Bt07,
    Bt1,
}

/// The bit pattern the preamble repeats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PreamblePattern {
    /// `0xAA` (`10101010`).
    Alternating10,
    /// `0x55` (`01010101`).
    Alternating01,
}

impl PreamblePattern {
    /// The byte that is repeated.
    pub const fn byte(self) -> u8 {
        match self {
            PreamblePattern::Alternating10 => 0xAA,
            PreamblePattern::Alternating01 => 0x55,
        }
    }
}

/// The sync word that follows the preamble, up to [`SyncWord::MAX_LEN`] bytes,
/// the first byte being sent first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncWord {
    bytes: [u8; Self::MAX_LEN],
    len: u8,
}

impl SyncWord {
    pub const MAX_LEN: usize = 8;

    /// A sync word of 1 to [`MAX_LEN`](Self::MAX_LEN) bytes, or `None` if `bytes` is empty or too long.
    pub const fn new(bytes: &[u8]) -> Option<Self> {
        if bytes.is_empty() || bytes.len() > Self::MAX_LEN {
            return None;
        }
        let mut word = Self { bytes: [0; Self::MAX_LEN], len: bytes.len() as u8 };
        let mut i = 0;
        while i < bytes.len() {
            word.bytes[i] = bytes[i];
            i += 1;
        }
        Some(word)
    }

    /// The sync word bytes, in transmit order.
    pub fn as_bytes(&self) -> &[u8] {
        // `len` could be wrong in data that came off the wire.
        &self.bytes[..(self.len as usize).min(Self::MAX_LEN)]
    }
}

/// How the receiver learns where the payload ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PacketLength {
    /// A length byte precedes the payload.
    Variable,
    /// Every packet has this many payload bytes.
    Fixed(u8),
}

/// The CRC that ends the packet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GfskCrc {
    None,
    /// A 16-bit CRC, sent most significant byte first.
    Crc16 {
        polynomial: u16,
        initial: u16,
        /// Whether the result is inverted (XORed with `0xFFFF`) before it is sent.
        invert_output: bool,
        /// Whether the length byte is covered as well as the payload.
        include_length: bool,
    },
}

impl GfskCrc {
    /// CRC-16/XMODEM (polynomial `0x1021`, initial value 0, no inversion).
    pub const fn xmodem(include_length: bool) -> Self {
        GfskCrc::Crc16 { polynomial: 0x1021, initial: 0x0000, invert_output: false, include_length }
    }
}

/// Coding applied to the packet bits after the CRC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataCoding {
    None,
    Manchester,
    Whitening,
}

/// The (G)FSK waveform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GfskConfig {
    /// Bit rate in bits/second.
    pub bit_rate: u32,
    /// Peak frequency deviation in Hz.
    pub freq_deviation: u32,
    pub pulse_shape: GfskPulseShape,
    pub preamble_pattern: PreamblePattern,
    /// Preamble length in bytes.
    pub preamble_length: u16,
    pub sync_word: SyncWord,
    pub length: PacketLength,
    pub crc: GfskCrc,
    pub coding: DataCoding,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_word_bounds() {
        assert_eq!(SyncWord::new(&[]), None);
        assert_eq!(SyncWord::new(&[0; 9]), None);
        assert_eq!(SyncWord::new(&[0x2D, 0xD4]).unwrap().as_bytes(), &[0x2D, 0xD4]);
        assert_eq!(SyncWord::new(&[1; 8]).unwrap().as_bytes(), &[1; 8]);
    }

    #[test]
    fn sync_words_compare_by_content() {
        assert_eq!(SyncWord::new(&[1, 2]), SyncWord::new(&[1, 2]));
        assert_ne!(SyncWord::new(&[1, 2]), SyncWord::new(&[1, 2, 0]));
    }

    #[test]
    fn preamble_bytes() {
        assert_eq!(PreamblePattern::Alternating10.byte(), 0xAA);
        assert_eq!(PreamblePattern::Alternating01.byte(), 0x55);
    }
}
