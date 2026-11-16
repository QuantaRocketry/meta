//! Turn a [`qcp::RadioConfig`] (the waveform on the air) into this chip's [`Config`].
//!
//! Receiver settings the waveform does not carry are chosen here: the channel
//! filter bandwidth from the bit rate and deviation, AGC on.

use qcp::{ConfigError, DataCoding, GfskConfig, GfskCrc, PacketLength, PhyConfig, PreamblePattern, RadioConfig};

use crate::{Config, RxBwMantissa};

/// Carrier range the HF-band setup in [`Sx1276::init`](crate::Sx1276::init) covers.
const FREQUENCY_HZ: core::ops::RangeInclusive<u32> = 862_000_000..=1_020_000_000;
const BIT_RATE_BPS: core::ops::RangeInclusive<u32> = 1_200..=300_000;
const MIN_FDEV_HZ: u32 = 600;
/// Datasheet limit on `fdev + bitrate / 2`.
const MAX_FDEV_PLUS_HALF_BIT_RATE_HZ: u32 = 250_000;
/// Widest channel filter the chip has with `exponent >= 1`.
const MAX_RX_BW_HZ: u32 = 250_000;
const FXOSC_HZ: u32 = 32_000_000;

/// The chip's own CRC: CCITT, init `0x1D0F`, inverted.
const HW_CRC: GfskCrc =
    GfskCrc::Crc16 { polynomial: 0x1021, initial: 0x1D0F, invert_output: true, include_length: false };

/// Smallest channel filter that passes `bitrate + 2 * fdev` with 50 % margin
/// for crystal and tuning error on both ends. Eggtimer (9600 baud, 35 kHz)
/// gets 125 kHz.
fn rx_bw(bit_rate: u32, fdev: u32) -> (RxBwMantissa, u8) {
    let needed = (bit_rate + 2 * fdev) * 3 / 2;
    let mut best = (MAX_RX_BW_HZ, RxBwMantissa::M16, 1);
    for exponent in 1..=7u8 {
        for (mantissa, divisor) in
            [(RxBwMantissa::M16, 16), (RxBwMantissa::M20, 20), (RxBwMantissa::M24, 24)]
        {
            let bw = FXOSC_HZ / (divisor << (exponent + 2));
            if bw >= needed && bw < best.0 {
                best = (bw, mantissa, exponent);
            }
        }
    }
    (best.1, best.2)
}

impl<'a> TryFrom<&'a RadioConfig> for Config<'a> {
    type Error = ConfigError;

    fn try_from(radio: &'a RadioConfig) -> Result<Self, ConfigError> {
        let PhyConfig::Gfsk(g) = &radio.phy else { return Err(ConfigError::UnsupportedModulation) };
        let GfskConfig { bit_rate, freq_deviation, .. } = *g;

        if !FREQUENCY_HZ.contains(&radio.frequency_hz) {
            return Err(ConfigError::FrequencyOutOfRange);
        }
        if !BIT_RATE_BPS.contains(&bit_rate) {
            return Err(ConfigError::BitRateOutOfRange);
        }
        if freq_deviation < MIN_FDEV_HZ
            || freq_deviation + bit_rate / 2 > MAX_FDEV_PLUS_HALF_BIT_RATE_HZ
        {
            return Err(ConfigError::DeviationOutOfRange);
        }
        if g.length != PacketLength::Variable {
            return Err(ConfigError::FixedLength);
        }
        if g.coding != DataCoding::None {
            return Err(ConfigError::DataCoding);
        }

        Ok(Config {
            frequency_hz: radio.frequency_hz,
            bitrate_bps: bit_rate,
            fdev_hz: freq_deviation,
            rx_bw: rx_bw(bit_rate, freq_deviation),
            sync_word: g.sync_word.as_bytes(),
            preamble_is_0x55: g.preamble_pattern == PreamblePattern::Alternating01,
            // Any other CRC (e.g. XMODEM) cannot be checked by the chip.
            hw_crc: g.crc == HW_CRC,
            agc_auto: true,
            lna_gain: 1,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qcp::{GfskPulseShape, LoraConfig, SyncWord};
    use qcp::config::radio::protocol::eggtimer::{self, Channel};

    fn eggtimer_config(center: u32, channel: Channel) -> RadioConfig {
        eggtimer::Protocol { center_frequency_hz: center, channel }.into()
    }

    #[test]
    fn eggtimer_maps_to_the_settings_proven_on_hardware() {
        let radio = eggtimer_config(925_000_000, Channel::Ch4);
        let cfg = Config::try_from(&radio).unwrap();
        assert_eq!(cfg.frequency_hz, 925_985_000);
        assert_eq!((cfg.bitrate_bps, cfg.fdev_hz), (9_600, 35_000));
        assert_eq!(cfg.rx_bw, (RxBwMantissa::M16, 2)); // 125 kHz
        assert_eq!(cfg.sync_word, &[0x2D, 0xD4]);
        assert!(cfg.preamble_is_0x55);
        assert!(!cfg.hw_crc);
        assert!(cfg.agc_auto);
    }

    #[test]
    fn rx_bw_is_the_smallest_filter_with_margin() {
        // 1.2 kbps, 5 kHz: needs 16.8 kHz; 32 MHz / (16 * 2^7) = 15.6 kHz is too narrow.
        assert_eq!(rx_bw(1_200, 5_000), (RxBwMantissa::M24, 4)); // 20.8 kHz
        assert_eq!(rx_bw(9_600, 35_000), (RxBwMantissa::M16, 2));
        assert_eq!(rx_bw(100_000, 50_000), (RxBwMantissa::M16, 1)); // 250 kHz
    }

    #[test]
    fn chip_native_crc_is_enabled() {
        let mut radio = eggtimer_config(925_000_000, Channel::Ch0);
        let PhyConfig::Gfsk(g) = &mut radio.phy else { unreachable!() };
        g.crc = HW_CRC;
        assert!(Config::try_from(&radio).unwrap().hw_crc);
    }

    #[test]
    fn unsupported_waveforms_are_rejected() {
        let ok = eggtimer_config(925_000_000, Channel::Ch0);
        let with = |f: &dyn Fn(&mut RadioConfig, &mut GfskConfig)| {
            let mut radio = ok;
            let PhyConfig::Gfsk(mut g) = radio.phy else { unreachable!() };
            f(&mut radio, &mut g);
            radio.phy = PhyConfig::Gfsk(g);
            Config::try_from(&radio).err()
        };
        assert_eq!(with(&|r, _| r.frequency_hz = 433_000_000), Some(ConfigError::FrequencyOutOfRange));
        assert_eq!(with(&|_, g| g.bit_rate = 0), Some(ConfigError::BitRateOutOfRange));
        assert_eq!(with(&|_, g| g.freq_deviation = 0), Some(ConfigError::DeviationOutOfRange));
        assert_eq!(with(&|_, g| g.freq_deviation = 300_000), Some(ConfigError::DeviationOutOfRange));
        assert_eq!(with(&|_, g| g.length = PacketLength::Fixed(8)), Some(ConfigError::FixedLength));
        assert_eq!(with(&|_, g| g.coding = DataCoding::Whitening), Some(ConfigError::DataCoding));
        assert_eq!(with(&|_, g| g.pulse_shape = GfskPulseShape::Bt05), None);
        assert_eq!(with(&|_, g| g.sync_word = SyncWord::new(&[1]).unwrap()), None);

        let lora = RadioConfig { frequency_hz: 915_000_000, phy: PhyConfig::Lora(LoraConfig::default()) };
        assert_eq!(Config::try_from(&lora).err(), Some(ConfigError::UnsupportedModulation));
    }
}
