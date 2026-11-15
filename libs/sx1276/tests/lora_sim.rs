//! Runs the LoRa driver against a simulated SX1276 (LoRa register page, addressed
//! FIFO, write-1-to-clear IRQ flags) and checks registers and data flow.

use core::convert::Infallible;
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{ErrorType as PinErrorType, OutputPin};
use embedded_hal::spi::{ErrorType, Operation, SpiDevice};
use std::cell::RefCell;
use std::rc::Rc;
use sx1276::lora::{Bandwidth, CodingRate, Config, Error, Header, Lora, SpreadingFactor, TxPower};

struct State {
    mem: [u8; 0x80],
    fifo: [u8; 256],
}

#[derive(Clone)]
struct SimChip(Rc<RefCell<State>>);

impl SimChip {
    fn new() -> Self {
        let mut mem = [0u8; 0x80];
        // Datasheet reset values of the registers the driver modifies.
        for (a, v) in [
            (0x01, 0x01), (0x09, 0x4F), (0x0B, 0x2B), (0x0C, 0x20), (0x1D, 0x72),
            (0x1E, 0x70), (0x26, 0x00), (0x31, 0xC3), (0x33, 0x27), (0x40, 0x00),
            (0x42, 0x12), (0x4D, 0x84),
        ] {
            mem[a] = v;
        }
        Self(Rc::new(RefCell::new(State { mem, fifo: [0; 256] })))
    }
    fn reg(&self, a: usize) -> u8 {
        self.0.borrow().mem[a]
    }
    fn set(&self, a: usize, v: u8) {
        self.0.borrow_mut().mem[a] = v;
    }
}

impl ErrorType for SimChip {
    type Error = Infallible;
}

impl SpiDevice for SimChip {
    fn transaction(&mut self, ops: &mut [Operation<'_, u8>]) -> Result<(), Infallible> {
        let mut st = self.0.borrow_mut();
        let mut addr = None;
        for op in ops {
            match op {
                Operation::Write(w) if addr.is_none() => addr = Some(w[0]),
                Operation::Write(w) => {
                    let a = addr.unwrap();
                    assert!(a & 0x80 != 0, "data written without the write bit");
                    let a = (a & 0x7F) as usize;
                    for (i, &b) in w.iter().enumerate() {
                        match a {
                            0 => {
                                let p = st.mem[0x0D] as usize;
                                st.fifo[p] = b;
                                st.mem[0x0D] = st.mem[0x0D].wrapping_add(1);
                            }
                            0x12 => st.mem[0x12] &= !b, // write 1 to clear
                            0x01 => {
                                let old = st.mem[0x01];
                                let mut new = b;
                                if old & 7 != 0 {
                                    new = (new & 0x7F) | (old & 0x80); // LongRangeMode only in Sleep
                                }
                                st.mem[0x01] = new;
                                if new & 7 == 3 {
                                    st.mem[0x12] |= 0x08; // TxDone
                                    st.mem[0x01] = (new & !7) | 1; // back to standby
                                }
                            }
                            _ => st.mem[a + i] = b,
                        }
                    }
                }
                Operation::Read(r) => {
                    let a = addr.unwrap();
                    assert!(a & 0x80 == 0, "read with the write bit set");
                    let a = a as usize;
                    for (i, b) in r.iter_mut().enumerate() {
                        *b = if a == 0 {
                            let p = st.mem[0x0D] as usize;
                            st.mem[0x0D] = st.mem[0x0D].wrapping_add(1);
                            st.fifo[p]
                        } else {
                            st.mem[a + i]
                        };
                    }
                }
                _ => unreachable!(),
            }
        }
        Ok(())
    }
}

struct Pin;
impl PinErrorType for Pin {
    type Error = Infallible;
}
impl OutputPin for Pin {
    fn set_low(&mut self) -> Result<(), Infallible> {
        Ok(())
    }
    fn set_high(&mut self) -> Result<(), Infallible> {
        Ok(())
    }
}

struct NoDelay;
impl DelayNs for NoDelay {
    fn delay_ns(&mut self, _: u32) {}
}

fn radio(cfg: &Config) -> (Lora<SimChip, Pin>, SimChip) {
    let chip = SimChip::new();
    let mut r = Lora::new(chip.clone(), Pin);
    r.init(&mut NoDelay, cfg).unwrap();
    (r, chip)
}

#[test]
fn init_programs_lora_registers() {
    let (_r, c) = radio(&Config::new(915_000_000));
    assert_eq!(c.reg(0x01), 0x81); // LoRa, standby
    assert_eq!([c.reg(0x06), c.reg(0x07), c.reg(0x08)], [0xE4, 0xC0, 0x00]); // 915 MHz
    assert_eq!(c.reg(0x09), 0xFB); // PA_BOOST, MaxPower 7, OutputPower 11 = 13 dBm
    assert_eq!(c.reg(0x4D) & 7, 4); // normal PaDac
    assert_eq!(c.reg(0x0C), 0x23); // LNA G1 + HF boost
    assert_eq!(c.reg(0x1D), 0x72); // 125 kHz, 4/5, explicit header
    assert_eq!(c.reg(0x1E), 0x74); // SF7, CRC on
    assert_eq!(c.reg(0x26), 0x04); // AGC on, LDRO off (1.0 ms symbols)
    assert_eq!([c.reg(0x20), c.reg(0x21)], [0x00, 0x08]); // 8-symbol preamble
    assert_eq!(c.reg(0x39), 0x12); // private sync word
    assert_eq!(c.reg(0x31) & 7, 3); // detection optimize SF7..12
    assert_eq!(c.reg(0x37), 0x0A);
    assert_eq!([c.reg(0x0E), c.reg(0x0F)], [0, 0]); // FIFO bases
    assert_eq!(c.reg(0x40) >> 6, 0); // DIO0 = RxDone
}

#[test]
fn sf12_enables_low_data_rate_optimize_automatically() {
    let mut cfg = Config::new(868_100_000);
    cfg.spreading_factor = SpreadingFactor::Sf12;
    let (_r, c) = radio(&cfg);
    assert_eq!(c.reg(0x26) & 0x08, 0x08); // 4096 chips / 125 kHz = 32.8 ms > 16 ms
    assert_eq!(c.reg(0x1E) >> 4, 12);
}

#[test]
fn sf6_needs_implicit_header_and_special_detection() {
    let mut cfg = Config::new(915_000_000);
    cfg.spreading_factor = SpreadingFactor::Sf6;
    let mut r = Lora::new(SimChip::new(), Pin);
    assert!(matches!(r.init(&mut NoDelay, &cfg), Err(Error::InvalidConfig(_))));

    cfg.header = Header::Implicit { payload_len: 8 };
    let (_r, c) = radio(&cfg);
    assert_eq!(c.reg(0x31) & 7, 5);
    assert_eq!(c.reg(0x37), 0x0C);
    assert_eq!(c.reg(0x1D) & 1, 1); // implicit header
    assert_eq!(c.reg(0x22), 8);
}

#[test]
fn rejects_invalid_configs_and_wrong_chip() {
    let mut cfg = Config::new(169_000_000);
    cfg.bandwidth = Bandwidth::Hz500000;
    let mut r = Lora::new(SimChip::new(), Pin);
    assert!(matches!(r.init(&mut NoDelay, &cfg), Err(Error::InvalidConfig(_))));

    for bad in [TxPower::PaBoost(20), TxPower::PaBoost(1), TxPower::Rfo(15), TxPower::Rfo(-1)] {
        let mut cfg = Config::new(915_000_000);
        cfg.tx_power = bad;
        let mut r = Lora::new(SimChip::new(), Pin);
        assert!(matches!(r.init(&mut NoDelay, &cfg), Err(Error::InvalidConfig(_))), "{bad:?}");
    }

    let chip = SimChip::new();
    chip.set(0x42, 0x00);
    let mut r = Lora::new(chip, Pin);
    assert!(matches!(r.init(&mut NoDelay, &Config::new(915_000_000)), Err(Error::BadVersion(0))));
}

#[test]
fn tx_power_options_set_pa_registers() {
    let mut cfg = Config::new(915_000_000);
    cfg.tx_power = TxPower::PaBoost20;
    let (_r, c) = radio(&cfg);
    assert_eq!(c.reg(0x09), 0xFF); // PA_BOOST, MaxPower 7, OutputPower 15
    assert_eq!(c.reg(0x4D) & 7, 7); // +20 dBm DAC
    assert_eq!(c.reg(0x0B), 0x20 | 17); // OCP on, 140 mA

    let mut cfg = Config::new(915_000_000);
    cfg.tx_power = TxPower::Rfo(10);
    let (_r, c) = radio(&cfg);
    assert_eq!(c.reg(0x09), 0x7A); // RFO, MaxPower 7, OutputPower 10
}

#[test]
fn rx_modes_set_up_fifo_and_irq() {
    let (mut r, c) = radio(&Config::new(915_000_000));
    c.set(0x0D, 0x55);
    c.set(0x12, 0xFF); // stale flags
    r.start_rx_continuous().unwrap();
    assert_eq!(c.reg(0x01) & 7, 5);
    assert_eq!(c.reg(0x0D), 0x00); // FIFO pointer at the Rx base
    assert_eq!(c.reg(0x12), 0x00); // flags cleared
    assert_eq!(c.reg(0x40) >> 6, 0); // RxDone on DIO0

    r.set_rx_timeout_symbols(0x123).unwrap();
    assert_eq!(c.reg(0x1E) & 3, 1);
    assert_eq!(c.reg(0x1F), 0x23);
    assert!(r.set_rx_timeout_symbols(2).is_err());
    r.start_rx_single().unwrap();
    assert_eq!(c.reg(0x01) & 7, 6);
}

#[test]
fn reads_a_packet_with_rssi_and_snr() {
    let (mut r, c) = radio(&Config::new(915_000_000));
    r.start_rx_continuous().unwrap();
    let mut buf = [0u8; 32];
    assert_eq!(r.read_packet(&mut buf).unwrap(), None);

    // A packet "hello" sits at 0x10 in the FIFO.
    c.0.borrow_mut().fifo[0x10..0x15].copy_from_slice(b"hello");
    c.set(0x10, 0x10); // FifoRxCurrentAddr
    c.set(0x13, 5); // RxNbBytes
    c.set(0x1A, 100); // PacketRssi: -157 + 100 = -57
    c.set(0x19, (-8i8) as u8); // SNR -2 dB
    c.set(0x12, 0x40 | 0x10); // RxDone | ValidHeader
    let p = r.read_packet(&mut buf).unwrap().unwrap();
    assert_eq!(&buf[..p.len], b"hello");
    assert_eq!((p.len, p.total_len), (5, 5));
    assert_eq!(p.snr_quarter_db, -8);
    assert_eq!(p.rssi_dbm, -59); // below noise floor: -57 + (-8 / 4)
    assert_eq!(c.reg(0x12), 0x00, "flags must be cleared");
    assert_eq!(r.read_packet(&mut buf).unwrap(), None);
}

#[test]
fn low_band_uses_the_lf_rssi_offset() {
    let (mut r, c) = radio(&Config::new(433_000_000));
    c.set(0x1B, 100);
    assert_eq!(r.rssi_dbm().unwrap(), -64); // -164 + 100
}

#[test]
fn crc_error_discards_the_packet() {
    let (mut r, c) = radio(&Config::new(915_000_000));
    c.set(0x12, 0x40 | 0x20); // RxDone | PayloadCrcError
    let mut buf = [0u8; 8];
    assert!(matches!(r.read_packet(&mut buf), Err(Error::PayloadCrc)));
    assert_eq!(c.reg(0x12), 0x00);
}

#[test]
fn oversized_packet_is_truncated_and_reported() {
    let (mut r, c) = radio(&Config::new(915_000_000));
    c.0.borrow_mut().fifo[..10].copy_from_slice(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    c.set(0x10, 0);
    c.set(0x13, 10);
    c.set(0x12, 0x40);
    let mut buf = [0u8; 4];
    let p = r.read_packet(&mut buf).unwrap().unwrap();
    assert_eq!((p.len, p.total_len), (4, 10));
    assert_eq!(buf, [0, 1, 2, 3]);
}

#[test]
fn implicit_header_uses_the_configured_length() {
    let mut cfg = Config::new(915_000_000);
    cfg.header = Header::Implicit { payload_len: 3 };
    cfg.coding_rate = CodingRate::Cr48;
    let (mut r, c) = radio(&cfg);
    assert_eq!(c.reg(0x1D) & 0x0F, (4 << 1) | 1);
    c.0.borrow_mut().fifo[..3].copy_from_slice(b"abc");
    c.set(0x13, 99); // ignored in implicit mode
    c.set(0x12, 0x40);
    let mut buf = [0u8; 8];
    let p = r.read_packet(&mut buf).unwrap().unwrap();
    assert_eq!((&buf[..p.len], p.total_len), (&b"abc"[..], 3));
}

#[test]
fn transmit_fills_fifo_and_waits_for_tx_done() {
    let (mut r, c) = radio(&Config::new(915_000_000));
    r.transmit(&mut NoDelay, b"ping", 100).unwrap();
    assert_eq!(&c.0.borrow().fifo[..4], b"ping");
    assert_eq!(c.reg(0x22), 4); // PayloadLength
    assert_eq!(c.reg(0x40) >> 6, 1); // TxDone on DIO0
    assert_eq!(c.reg(0x12) & 0x08, 0, "TxDone flag cleared");
    assert!(matches!(r.mode().unwrap(), sx1276::lora::LoraMode::Standby));
    assert!(matches!(r.start_transmit(&[]), Err(Error::PayloadSize)));
    assert!(matches!(r.start_transmit(&[0u8; 256]), Err(Error::PayloadSize)));
}

#[test]
fn receive_after_transmit_restores_implicit_length() {
    let mut cfg = Config::new(915_000_000);
    cfg.header = Header::Implicit { payload_len: 7 };
    let (mut r, c) = radio(&cfg);
    r.transmit(&mut NoDelay, b"xy", 10).unwrap();
    assert_eq!(c.reg(0x22), 2);
    r.start_rx_continuous().unwrap();
    assert_eq!(c.reg(0x22), 7);
}

#[test]
fn cad_reports_detection() {
    let (mut r, c) = radio(&Config::new(915_000_000));
    r.start_cad().unwrap();
    assert_eq!(c.reg(0x40) >> 6, 2); // CadDone on DIO0
    assert_eq!(r.cad_result().unwrap(), None);
    c.set(0x12, 0x04 | 0x01); // CadDone | CadDetected
    assert_eq!(r.cad_result().unwrap(), Some(true));
    assert_eq!(c.reg(0x12), 0);
}

#[test]
fn release_returns_the_bus() {
    let (r, _c) = radio(&Config::new(915_000_000));
    let (_spi, _rst) = r.release();
}
