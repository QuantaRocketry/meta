use serde::{Deserialize, Serialize};

use crate::config::radio::{
    DataCoding, GfskConfig, GfskCrc, GfskPulseShape, PacketLength, PhyConfig, PreamblePattern,
    RadioConfig, SyncWord,
};

/// Default centre frequency, as used by the eggtimer decoder.
pub const DEFAULT_CENTER_FREQUENCY_HZ: u32 = 921_000_000;

/// Eggtimer TX bit rate in bits/second.
pub const BIT_RATE: u32 = 9_600;
/// Eggtimer TX peak frequency deviation in Hz (measured at about +/-34.5 kHz).
pub const FREQ_DEVIATION_HZ: u32 = 35_000;
/// Preamble length in bytes (`0x55` repeated).
pub const PREAMBLE_BYTES: u16 = 5;
/// The sync word that follows the preamble.
pub const SYNC_WORD: SyncWord = match SyncWord::new(&[0x2D, 0xD4]) {
    Some(word) => word,
    None => panic!("invalid sync word"),
};

/// An Eggtimer / Eggfinder TX link.
///
/// A transmitter sits on one of eight [`Channel`]s. The carrier frequency is
/// the centre frequency plus the channel's offset (see
/// [`carrier_frequency_hz`](Self::carrier_frequency_hz)); that is the
/// frequency a receiver has to be tuned to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Protocol {
    /// Centre frequency in Hz that the channel offsets are applied to.
    pub center_frequency_hz: u32,
    pub channel: Channel,
}

impl Default for Protocol {
    fn default() -> Self {
        Self {
            center_frequency_hz: DEFAULT_CENTER_FREQUENCY_HZ,
            channel: Channel::default(),
        }
    }
}

impl Protocol {
    /// Frequency in Hz the transmitter is on: the centre frequency plus the
    /// channel's offset (e.g. 925 MHz and channel 4 give 925.985 MHz).
    pub const fn carrier_frequency_hz(&self) -> u32 {
        self.center_frequency_hz.saturating_add_signed(self.channel.offset_hz())
    }
}

/// The waveform of an eggtimer transmitter: its carrier frequency (centre plus
/// channel offset) and its modulation. The frame is a `0x55` preamble, the sync
/// word `2D D4`, a length byte, the payload and a CRC-16/XMODEM that covers the
/// length byte and the payload.
impl From<Protocol> for RadioConfig {
    fn from(protocol: Protocol) -> Self {
        Self {
            frequency_hz: protocol.carrier_frequency_hz(),
            phy: PhyConfig::Gfsk(GfskConfig {
                bit_rate: BIT_RATE,
                freq_deviation: FREQ_DEVIATION_HZ,
                // Plain 2-FSK; no transmit shaping is known.
                pulse_shape: GfskPulseShape::NoFilter,
                preamble_pattern: PreamblePattern::Alternating01,
                preamble_length: PREAMBLE_BYTES,
                sync_word: SYNC_WORD,
                length: PacketLength::Variable,
                crc: GfskCrc::xmodem(true),
                coding: DataCoding::None,
            }),
        }
    }
}

/// Eggtimer channel number, 0 to 7.
///
/// Channel 0 sits 15 kHz below the centre frequency (measured on a real
/// transmitter). Channels 1 to 7 follow it at 250 kHz spacing, starting
/// 235 kHz above the centre; those offsets are inferred, not measured.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Channel {
    #[default]
    Ch0,
    Ch1,
    Ch2,
    Ch3,
    Ch4,
    Ch5,
    Ch6,
    Ch7,
}

impl Channel {
    /// Offset in Hz of this channel from the centre frequency.
    pub const fn offset_hz(self) -> i32 {
        match self {
            Channel::Ch0 => -15_000,
            ch => 235_000 + (ch.number() as i32 - 1) * 250_000,
        }
    }

    /// The channel number, 0 to 7.
    pub const fn number(self) -> u8 {
        self as u8
    }

    /// The channel with the given number, if it is 0 to 7.
    pub const fn from_number(number: u8) -> Option<Self> {
        Some(match number {
            0 => Channel::Ch0,
            1 => Channel::Ch1,
            2 => Channel::Ch2,
            3 => Channel::Ch3,
            4 => Channel::Ch4,
            5 => Channel::Ch5,
            6 => Channel::Ch6,
            7 => Channel::Ch7,
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_round_trip;
    use crate::config::radio::Protocol as RadioProtocol;

    #[test]
    fn default_matches_the_decoder() {
        let p = Protocol::default();
        assert_eq!(p.center_frequency_hz, 921_000_000);
        assert_eq!(p.channel, Channel::Ch0);
        assert_eq!(p.carrier_frequency_hz(), 920_985_000);
    }

    #[test]
    fn channel_offsets() {
        let offsets: [i32; 8] = core::array::from_fn(|n| {
            Channel::from_number(n as u8).unwrap().offset_hz()
        });
        assert_eq!(
            offsets,
            [-15_000, 235_000, 485_000, 735_000, 985_000, 1_235_000, 1_485_000, 1_735_000]
        );
    }

    #[test]
    fn channel_numbers_round_trip() {
        for n in 0..8 {
            assert_eq!(Channel::from_number(n).unwrap().number(), n);
        }
        assert_eq!(Channel::from_number(8), None);
    }

    #[test]
    fn carrier_is_centre_plus_channel_offset() {
        let p = Protocol { center_frequency_hz: 925_000_000, channel: Channel::Ch4 };
        assert_eq!(p.carrier_frequency_hz(), 925_985_000);
    }

    #[test]
    fn radio_config_is_the_eggtimer_waveform_at_the_carrier_frequency() {
        let p = Protocol { center_frequency_hz: 925_000_000, channel: Channel::Ch4 };
        let radio: RadioConfig = p.into();
        assert_eq!(radio.frequency_hz, 925_985_000);
        let PhyConfig::Gfsk(g) = radio.phy else { panic!("expected GFSK, got {:?}", radio.phy) };
        assert_eq!((g.bit_rate, g.freq_deviation), (9_600, 35_000));
        assert_eq!(g.pulse_shape, GfskPulseShape::NoFilter);
        assert_eq!((g.preamble_pattern.byte(), g.preamble_length), (0x55, 5));
        assert_eq!(g.sync_word.as_bytes(), &[0x2D, 0xD4]);
        assert_eq!(g.length, PacketLength::Variable);
        assert_eq!(g.crc, GfskCrc::xmodem(true));
        assert_eq!(g.coding, DataCoding::None);
    }

    #[test]
    fn default_protocol_gives_the_default_carrier() {
        let radio: RadioConfig = Protocol::default().into();
        assert_eq!(radio.frequency_hz, 920_985_000);
    }

    #[test]
    fn only_the_frequency_depends_on_the_channel() {
        let a: RadioConfig = Protocol::default().into();
        let b: RadioConfig =
            Protocol { center_frequency_hz: 921_000_000, channel: Channel::Ch7 }.into();
        assert_ne!(a.frequency_hz, b.frequency_hz);
        assert_eq!(a.phy, b.phy);
    }

    #[test]
    fn non_default_round_trip() {
        let p = Protocol { center_frequency_hz: 925_000_000, channel: Channel::Ch7 };
        assert_round_trip(&RadioProtocol::Eggtimer(p).into());
    }
}
