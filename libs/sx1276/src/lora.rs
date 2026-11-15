//! LoRa driver for the SX1276/77/78/79 (RFM95/96/97/98).
//!
//! Follows the datasheet's LoRa flows (Rev. 4, section 4.1.6): configuration is
//! written in Standby, receive data is read through the addressed FIFO, and
//! `DIO0` signals `RxDone` (Rx) / `TxDone` (Tx) / `CadDone` (CAD).
//!
//! Not covered: frequency hopping, the 500 kHz bandwidth sensitivity
//! workaround from Semtech's errata (registers not described in the datasheet),
//! and RSSI calibration for front-ends other than the plain HF/LF ports.

use device_driver::{BufferInterface, RegisterInterface};
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use embedded_hal::spi::SpiDevice;

use crate::lora_registers::LoraRegisters;
pub use crate::lora_registers::{Bandwidth, CodingRate, LnaGain, LoraMode};
use crate::SpiInterface;

/// Crystal frequency (Hz).
const FXOSC: u64 = 32_000_000;
/// Silicon version of every SX1276/77/78/79.
const EXPECTED_VERSION: u8 = 0x12;
/// Carrier frequencies below this use the low-frequency port (RSSI offset -164 instead of -157).
const LF_BAND_MAX_HZ: u32 = 525_000_000;
/// Rx and Tx both use the whole 256-byte FIFO, starting at 0 (datasheet 4.1.2.3).
const FIFO_BASE: u8 = 0x00;

/// Everything that can go wrong.
#[derive(Debug)]
pub enum Error<E> {
    /// SPI bus error.
    Spi(E),
    /// Reset pin error.
    Pin,
    /// `RegVersion` did not read 0x12: wiring problem or wrong chip.
    BadVersion(u8),
    /// `LongRangeMode` did not stick after the switch from Sleep.
    LoraModeNotSet,
    /// The chip did not enter the requested mode.
    ModeTimeout { target: LoraMode, mode: LoraMode },
    /// A configuration value is invalid or unsupported (the text says which).
    InvalidConfig(&'static str),
    /// The received packet failed its payload CRC. Its data is discarded.
    PayloadCrc,
    /// The payload must be 1..=255 bytes.
    PayloadSize,
    /// `TxDone` never came.
    TxTimeout,
}

impl<E> From<E> for Error<E> {
    fn from(e: E) -> Self {
        Error::Spi(e)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpreadingFactor {
    /// 64 chips/symbol. Requires implicit header mode.
    Sf6 = 6,
    Sf7 = 7,
    Sf8 = 8,
    Sf9 = 9,
    Sf10 = 10,
    Sf11 = 11,
    Sf12 = 12,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Header {
    /// The packet carries length, coding rate and CRC flag.
    Explicit,
    /// No header: both ends must agree on length, coding rate and CRC setting.
    Implicit { payload_len: u8 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TxPower {
    /// RFO pin, 0..=14 dBm.
    Rfo(i8),
    /// PA_BOOST pin, 2..=17 dBm. (RFM95 modules are wired to PA_BOOST.)
    PaBoost(i8),
    /// PA_BOOST pin with the +20 dBm option (PaDac 0x07, over-current limit raised to 140 mA).
    PaBoost20,
}

/// Modem settings. Both ends of a link must agree on all of these (except
/// power, AGC and LNA gain).
#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub frequency_hz: u32,
    pub bandwidth: Bandwidth,
    pub spreading_factor: SpreadingFactor,
    pub coding_rate: CodingRate,
    pub header: Header,
    /// Payload CRC (generated on Tx, checked on Rx; in explicit mode Rx follows the header).
    pub crc: bool,
    /// Preamble length in symbols (4.25 more are sent on air).
    pub preamble_symbols: u16,
    /// 0x12 = private network (default), 0x34 = LoRaWAN.
    pub sync_word: u8,
    pub invert_iq: bool,
    /// `None` = enable automatically when the symbol time exceeds 16 ms, as the datasheet mandates.
    pub low_data_rate_optimize: Option<bool>,
    /// LNA gain from the AGC loop. When `false`, `lna_gain` is a fixed gain.
    pub agc_auto: bool,
    pub lna_gain: LnaGain,
    pub tx_power: TxPower,
}

impl Config {
    /// Sensible defaults: 125 kHz, SF7, 4/5, explicit header, CRC on, 8-symbol preamble,
    /// private sync word, AGC on, +13 dBm on PA_BOOST.
    pub const fn new(frequency_hz: u32) -> Self {
        Self {
            frequency_hz,
            bandwidth: Bandwidth::Hz125000,
            spreading_factor: SpreadingFactor::Sf7,
            coding_rate: CodingRate::Cr45,
            header: Header::Explicit,
            crc: true,
            preamble_symbols: 8,
            sync_word: 0x12,
            invert_iq: false,
            low_data_rate_optimize: None,
            agc_auto: true,
            lna_gain: LnaGain::G1,
            tx_power: TxPower::PaBoost(13),
        }
    }
}

/// A received packet. The data is in the buffer given to [`Lora::read_packet`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Packet {
    /// Bytes copied into the buffer.
    pub len: usize,
    /// Payload length announced by the packet (`> len` if the buffer was too small).
    pub total_len: usize,
    /// Packet strength in dBm (datasheet 5.5.5, including the SNR < 0 correction).
    pub rssi_dbm: i16,
    /// SNR in quarter dB (divide by 4).
    pub snr_quarter_db: i8,
}

/// SX1276 in LoRa mode. `SPI` must assert chip select for the whole of each
/// transaction (e.g. `embedded_hal_bus::spi::ExclusiveDevice`).
pub struct Lora<SPI: SpiDevice, RST> {
    dev: LoraRegisters<SpiInterface<SPI>>,
    rst: RST,
    frequency_hz: u32,
    /// `Some(len)` in implicit header mode.
    implicit_len: Option<u8>,
}

fn bandwidth_hz<E>(bw: Bandwidth) -> Result<u32, Error<E>> {
    Ok(match bw {
        Bandwidth::Hz7800 => 7_800,
        Bandwidth::Hz10400 => 10_400,
        Bandwidth::Hz15600 => 15_600,
        Bandwidth::Hz20800 => 20_800,
        Bandwidth::Hz31250 => 31_250,
        Bandwidth::Hz41700 => 41_700,
        Bandwidth::Hz62500 => 62_500,
        Bandwidth::Hz125000 => 125_000,
        Bandwidth::Hz250000 => 250_000,
        Bandwidth::Hz500000 => 500_000,
        _ => return Err(Error::InvalidConfig("reserved bandwidth value")),
    })
}

impl<SPI: SpiDevice, RST: OutputPin> Lora<SPI, RST> {
    pub fn new(spi: SPI, rst: RST) -> Self {
        Self {
            dev: LoraRegisters::new(SpiInterface::new(spi)),
            rst,
            frequency_hz: 0,
            implicit_len: None,
        }
    }

    /// Give the SPI device and reset pin back, e.g. to switch to FSK ([`crate::Sx1276`]).
    pub fn release(self) -> (SPI, RST) {
        (self.dev.interface.into_inner(), self.rst)
    }

    /// Direct access to the generated register map.
    pub fn registers(&mut self) -> &mut LoraRegisters<SpiInterface<SPI>> {
        &mut self.dev
    }

    /// Request `target`, without waiting. Use for Tx and CAD, which end on their own.
    fn request_mode(&mut self, target: LoraMode) -> Result<(), SPI::Error> {
        self.dev.op_mode().modify(|r| r.set_mode(target))
    }

    /// Request `target` and wait until `RegOpMode` reads it back.
    fn enter_mode(&mut self, target: LoraMode) -> Result<(), Error<SPI::Error>> {
        self.request_mode(target)?;
        let mut mode = target;
        for _ in 0..1_000 {
            mode = self.dev.op_mode().read()?.mode();
            if core::mem::discriminant(&mode) == core::mem::discriminant(&target) {
                return Ok(());
            }
        }
        Err(Error::ModeTimeout { target, mode })
    }

    fn clear_irq_flags(&mut self) -> Result<(), SPI::Error> {
        self.dev.irq_flags().write(|r| {
            r.set_rx_timeout(true);
            r.set_rx_done(true);
            r.set_payload_crc_error(true);
            r.set_valid_header(true);
            r.set_tx_done(true);
            r.set_cad_done(true);
            r.set_fhss_change_channel(true);
            r.set_cad_detected(true);
        })
    }

    fn rssi_offset(&self) -> i16 {
        if self.frequency_hz < LF_BAND_MAX_HZ { -164 } else { -157 }
    }

    /// Hardware-reset the module, switch it to LoRa and apply `cfg`. Leaves the radio in standby.
    pub fn init(&mut self, delay: &mut impl DelayNs, cfg: &Config) -> Result<(), Error<SPI::Error>> {
        let bw_hz = bandwidth_hz(cfg.bandwidth)?;
        let sf = cfg.spreading_factor as u8;
        if sf == 6 && !matches!(cfg.header, Header::Implicit { .. }) {
            return Err(Error::InvalidConfig("SF6 requires implicit header mode"));
        }
        if matches!(cfg.header, Header::Implicit { payload_len: 0 }) {
            return Err(Error::InvalidConfig("implicit payload length must be 1..=255"));
        }
        if matches!(cfg.coding_rate, CodingRate::Reserved0 | CodingRate::Reserved5 | CodingRate::Reserved6 | CodingRate::Reserved7) {
            return Err(Error::InvalidConfig("reserved coding rate"));
        }
        if matches!(cfg.bandwidth, Bandwidth::Hz250000 | Bandwidth::Hz500000) && cfg.frequency_hz < 410_000_000 {
            return Err(Error::InvalidConfig("250/500 kHz bandwidth is not supported in the 169 MHz band"));
        }

        // Reset: pull low >100 us, release, wait >=5 ms.
        self.rst.set_low().map_err(|_| Error::Pin)?;
        delay.delay_ms(1);
        self.rst.set_high().map_err(|_| Error::Pin)?;
        delay.delay_ms(10);

        let version = self.dev.version().read()?.value();
        if version != EXPECTED_VERSION {
            return Err(Error::BadVersion(version));
        }

        // LongRangeMode can only be changed in Sleep: go to Sleep first, then set it.
        self.dev.op_mode().modify(|r| r.set_mode(LoraMode::Sleep))?;
        delay.delay_ms(2);
        self.dev.op_mode().modify(|r| {
            r.set_long_range_mode(true);
            r.set_access_shared_reg(false);
            r.set_mode(LoraMode::Sleep);
        })?;
        delay.delay_ms(2);
        if !self.dev.op_mode().read()?.long_range_mode() {
            return Err(Error::LoraModeNotSet);
        }
        self.enter_mode(LoraMode::Standby)?;

        // Carrier = value * Fstep.
        self.frequency_hz = cfg.frequency_hz;
        self.dev.frf().write(|r| r.set_value((((cfg.frequency_hz as u64) << 19) / FXOSC) as u32))?;

        self.set_tx_power(cfg.tx_power)?;

        // LNA: gain (ignored while AGC is on), boost on for the HF port.
        self.dev.lna().modify(|r| {
            r.set_lna_gain(cfg.lna_gain);
            r.set_lna_boost_hf(if cfg.frequency_hz >= LF_BAND_MAX_HZ { 0b11 } else { 0b00 });
        })?;

        let implicit = matches!(cfg.header, Header::Implicit { .. });
        self.implicit_len = match cfg.header {
            Header::Implicit { payload_len } => Some(payload_len),
            Header::Explicit => None,
        };
        self.dev.modem_config_1().modify(|r| {
            r.set_bandwidth(cfg.bandwidth);
            r.set_coding_rate(cfg.coding_rate);
            r.set_implicit_header(implicit);
        })?;
        self.dev.modem_config_2().modify(|r| {
            r.set_spreading_factor(sf);
            r.set_tx_continuous(false);
            r.set_rx_payload_crc_on(cfg.crc);
        })?;
        // Low data rate optimisation: mandated when the symbol time exceeds 16 ms.
        let symbol_us = ((1u64 << sf) * 1_000_000 / bw_hz as u64) as u32;
        let ldro = cfg.low_data_rate_optimize.unwrap_or(symbol_us > 16_000);
        self.dev.modem_config_3().modify(|r| {
            r.set_low_data_rate_optimize(ldro);
            r.set_agc_auto_on(cfg.agc_auto);
        })?;

        self.dev.preamble_length().write(|r| r.set_value(cfg.preamble_symbols))?;
        self.dev.sync_word().write(|r| r.set_value(cfg.sync_word))?;
        self.dev.invert_iq().modify(|r| r.set_invert_iq(cfg.invert_iq))?;

        // SF6 needs special detection settings; every other SF the defaults (datasheet 4.1.1.2).
        self.dev.detect_optimize().modify(|r| r.set_detection_optimize(if sf == 6 { 0b101 } else { 0b011 }))?;
        self.dev.detection_threshold().write(|r| r.set_value(if sf == 6 { 0x0C } else { 0x0A }))?;

        self.dev.max_payload_length().write(|r| r.set_value(0xFF))?;
        if let Some(len) = self.implicit_len {
            self.dev.payload_length().write(|r| r.set_value(len))?;
        }
        self.dev.fifo_tx_base_addr().write(|r| r.set_value(FIFO_BASE))?;
        self.dev.fifo_rx_base_addr().write(|r| r.set_value(FIFO_BASE))?;

        // All interrupts enabled; DIO0 = RxDone.
        self.dev.irq_flags_mask().write(|_| {})?;
        self.dev.dio_mapping_1().modify(|r| r.set_dio_0(0b00))?;
        self.clear_irq_flags()?;
        Ok(())
    }

    /// Change the transmit power (Sleep/Standby only).
    pub fn set_tx_power(&mut self, power: TxPower) -> Result<(), Error<SPI::Error>> {
        let (pa_boost, output_power, dac, ocp_trim) = match power {
            TxPower::Rfo(p) if (0..=14).contains(&p) => (false, p as u8, 0x04, 0x0B),
            TxPower::PaBoost(p) if (2..=17).contains(&p) => (true, (p - 2) as u8, 0x04, 0x0B),
            TxPower::PaBoost20 => (true, 15, 0x07, 17), // 17 -> 140 mA
            _ => return Err(Error::InvalidConfig("tx power out of range")),
        };
        // MaxPower 7: Pmax = 10.8 + 0.6 * 7 = 15 dBm, so for RFO Pout = OutputPower.
        self.dev.pa_config().modify(|r| {
            r.set_pa_select(pa_boost);
            r.set_max_power(7);
            r.set_output_power(output_power);
        })?;
        self.dev.pa_dac().modify(|r| r.set_pa_dac(dac))?;
        self.dev.ocp().modify(|r| {
            r.set_ocp_on(true);
            r.set_ocp_trim(ocp_trim);
        })?;
        Ok(())
    }

    /// Rx single mode: how long to wait for a preamble, in symbols (4..=1023).
    pub fn set_rx_timeout_symbols(&mut self, symbols: u16) -> Result<(), Error<SPI::Error>> {
        if !(4..=1023).contains(&symbols) {
            return Err(Error::InvalidConfig("rx timeout must be 4..=1023 symbols"));
        }
        self.dev.modem_config_2().modify(|r| r.set_symb_timeout_msb((symbols >> 8) as u8))?;
        self.dev.symb_timeout_lsb().write(|r| r.set_value(symbols as u8))?;
        Ok(())
    }

    fn prepare_rx(&mut self) -> Result<(), Error<SPI::Error>> {
        self.enter_mode(LoraMode::Standby)?;
        if let Some(len) = self.implicit_len {
            // A transmit may have changed it.
            self.dev.payload_length().write(|r| r.set_value(len))?;
        }
        self.dev.fifo_addr_ptr().write(|r| r.set_value(FIFO_BASE))?;
        self.dev.dio_mapping_1().modify(|r| r.set_dio_0(0b00))?; // RxDone
        self.clear_irq_flags()?;
        Ok(())
    }

    /// Receive continuously. DIO0 rises on `RxDone`; then call [`Self::read_packet`].
    pub fn start_rx_continuous(&mut self) -> Result<(), Error<SPI::Error>> {
        self.prepare_rx()?;
        self.enter_mode(LoraMode::RxContinuous)
    }

    /// Receive one packet, or time out (see [`Self::set_rx_timeout_symbols`]) and
    /// return to standby. `RxDone` or `RxTimeout` is signalled.
    pub fn start_rx_single(&mut self) -> Result<(), Error<SPI::Error>> {
        self.prepare_rx()?;
        self.enter_mode(LoraMode::RxSingle)
    }

    /// `true` if an Rx single window ended without a packet. Clears the flag.
    pub fn rx_timed_out(&mut self) -> Result<bool, Error<SPI::Error>> {
        if self.dev.irq_flags().read()?.rx_timeout() {
            self.dev.irq_flags().write(|r| r.set_rx_timeout(true))?;
            return Ok(true);
        }
        Ok(false)
    }

    /// If a packet has been received, copy it into `buf`.
    ///
    /// Returns `Ok(None)` while nothing is pending and `Err(Error::PayloadCrc)`
    /// (packet discarded) if its CRC was bad. In continuous mode call this
    /// before the next `RxDone`: the status registers only describe the last packet.
    pub fn read_packet(&mut self, buf: &mut [u8]) -> Result<Option<Packet>, Error<SPI::Error>> {
        let flags = self.dev.irq_flags().read()?;
        if !flags.rx_done() {
            return Ok(None);
        }
        let crc_error = flags.payload_crc_error();
        self.dev.irq_flags().write(|r| {
            r.set_rx_done(true);
            r.set_payload_crc_error(true);
            r.set_valid_header(true);
        })?;
        if crc_error {
            return Err(Error::PayloadCrc);
        }

        let total = match self.implicit_len {
            Some(len) => len,
            None => self.dev.rx_nb_bytes().read()?.value(),
        } as usize;
        // Point at the start of the packet just received, then stream it out.
        let start = self.dev.fifo_rx_current_addr().read()?.value();
        self.dev.fifo_addr_ptr().write(|r| r.set_value(start))?;
        let len = total.min(buf.len());
        if len > 0 {
            self.dev.fifo().read(&mut buf[..len])?;
        }

        let snr = self.dev.pkt_snr_value().read()?.value();
        let mut rssi = self.rssi_offset() + self.dev.pkt_rssi_value().read()?.value() as i16;
        if snr < 0 {
            rssi += snr as i16 / 4; // below the noise floor: add SNR (quarter dB)
        }
        Ok(Some(Packet { len, total_len: total, rssi_dbm: rssi, snr_quarter_db: snr }))
    }

    /// Start sending `payload` (1..=255 bytes). DIO0 rises on `TxDone`; poll [`Self::tx_done`].
    pub fn start_transmit(&mut self, payload: &[u8]) -> Result<(), Error<SPI::Error>> {
        if payload.is_empty() || payload.len() > 255 {
            return Err(Error::PayloadSize);
        }
        // The FIFO can only be filled in Standby.
        self.enter_mode(LoraMode::Standby)?;
        self.dev.fifo_addr_ptr().write(|r| r.set_value(FIFO_BASE))?;
        self.dev.fifo().write_all(payload)?;
        self.dev.payload_length().write(|r| r.set_value(payload.len() as u8))?;
        self.dev.dio_mapping_1().modify(|r| r.set_dio_0(0b01))?; // TxDone
        self.clear_irq_flags()?;
        // Tx returns to standby by itself, so reading the mode back proves nothing.
        self.request_mode(LoraMode::Tx)?;
        Ok(())
    }

    /// `true` once the packet started by [`Self::start_transmit`] has been sent. Clears the flag.
    pub fn tx_done(&mut self) -> Result<bool, Error<SPI::Error>> {
        if self.dev.irq_flags().read()?.tx_done() {
            self.dev.irq_flags().write(|r| r.set_tx_done(true))?;
            return Ok(true);
        }
        Ok(false)
    }

    /// Send `payload` and wait for `TxDone`, polling every millisecond.
    /// Time on air grows quickly with the spreading factor, so choose `timeout_ms` accordingly.
    pub fn transmit(
        &mut self,
        delay: &mut impl DelayNs,
        payload: &[u8],
        timeout_ms: u32,
    ) -> Result<(), Error<SPI::Error>> {
        self.start_transmit(payload)?;
        for _ in 0..timeout_ms {
            if self.tx_done()? {
                return Ok(());
            }
            delay.delay_ms(1);
        }
        let _ = self.enter_mode(LoraMode::Standby);
        Err(Error::TxTimeout)
    }

    /// Channel activity detection: look for a LoRa preamble. DIO0 rises on `CadDone`;
    /// poll [`Self::cad_result`].
    pub fn start_cad(&mut self) -> Result<(), Error<SPI::Error>> {
        self.enter_mode(LoraMode::Standby)?;
        self.dev.dio_mapping_1().modify(|r| r.set_dio_0(0b10))?; // CadDone
        self.clear_irq_flags()?;
        self.request_mode(LoraMode::Cad)?;
        Ok(())
    }

    /// `None` while CAD is running, then `Some(true)` if a preamble was detected. Clears the flags.
    pub fn cad_result(&mut self) -> Result<Option<bool>, Error<SPI::Error>> {
        let f = self.dev.irq_flags().read()?;
        if !f.cad_done() {
            return Ok(None);
        }
        let detected = f.cad_detected();
        self.dev.irq_flags().write(|r| {
            r.set_cad_done(true);
            r.set_cad_detected(true);
        })?;
        Ok(Some(detected))
    }

    pub fn standby(&mut self) -> Result<(), Error<SPI::Error>> {
        self.enter_mode(LoraMode::Standby)
    }

    pub fn sleep(&mut self) -> Result<(), Error<SPI::Error>> {
        self.enter_mode(LoraMode::Sleep)
    }

    /// Current operating mode.
    pub fn mode(&mut self) -> Result<LoraMode, Error<SPI::Error>> {
        Ok(self.dev.op_mode().read()?.mode())
    }

    /// Current RSSI in dBm (valid while receiving; average it for precision).
    pub fn rssi_dbm(&mut self) -> Result<i16, Error<SPI::Error>> {
        Ok(self.rssi_offset() + self.dev.rssi_value().read()?.value() as i16)
    }
}

#[allow(unused)]
fn _assert_interface_traits<SPI: SpiDevice>() {
    fn needs<T: RegisterInterface + BufferInterface>() {}
    needs::<SpiInterface<SPI>>();
}
