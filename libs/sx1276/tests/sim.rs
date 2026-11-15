//! Runs the driver against a simulated SX1276 register file and checks the
//! bytes that end up in the registers.

use core::convert::Infallible;
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{ErrorType as PinErrorType, OutputPin};
use embedded_hal::spi::{ErrorType, Operation, SpiDevice};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use sx1276::{Config, Mode, RxBwMantissa, Sx1276};

struct State {
    mem: [u8; 0x80],
    fifo: VecDeque<u8>,
}

/// Simulated chip; the test keeps a second handle to inspect the registers.
#[derive(Clone)]
struct SimChip(Rc<RefCell<State>>);

impl SimChip {
    fn new() -> Self {
        let mut mem = [0u8; 0x80];
        mem[0x42] = 0x12; // version
        mem[0x3E] = 0x90; // ModeReady | PllLock: every mode switch "completes"
        // Datasheet reset values of the registers the driver modifies.
        for (a, v) in [
            (0x01, 0x01), (0x09, 0x4F), (0x0C, 0x20), (0x0D, 0x08), (0x12, 0x15),
            (0x13, 0x0B), (0x1F, 0xAA), (0x27, 0x93), (0x30, 0x90), (0x31, 0x40),
            (0x35, 0x0F),
        ] {
            mem[a] = v;
        }
        Self(Rc::new(RefCell::new(State { mem, fifo: VecDeque::new() })))
    }
    fn reg(&self, a: usize) -> u8 {
        self.0.borrow().mem[a]
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
                    for (i, b) in w.iter().enumerate() {
                        if a == 0 {
                            st.fifo.push_back(*b);
                        } else {
                            st.mem[a + i] = *b;
                        }
                    }
                }
                Operation::Read(r) => {
                    let a = addr.unwrap();
                    assert!(a & 0x80 == 0, "read with the write bit set");
                    let a = a as usize;
                    for (i, b) in r.iter_mut().enumerate() {
                        *b = if a == 0 { st.fifo.pop_front().unwrap_or(0) } else { st.mem[a + i] };
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

const CFG: Config = Config {
    frequency_hz: 925_985_000, // 925 MHz centre + channel 4 offset (985 kHz)
    bitrate_bps: 9_600,
    fdev_hz: 35_000,
    rx_bw: (RxBwMantissa::M16, 2), // 125 kHz
    sync_word: &[0x2D, 0xD4],
    preamble_is_0x55: true,
    hw_crc: false,
    agc_auto: true,
    lna_gain: 1,
};

fn radio() -> (Sx1276<SimChip, Pin>, SimChip) {
    let chip = SimChip::new();
    let mut r = Sx1276::new(chip.clone(), Pin);
    r.init(&mut NoDelay, &CFG).unwrap();
    (r, chip)
}

#[test]
fn init_programs_eggtimer_channel_4() {
    let (_r, c) = radio();
    assert_eq!([c.reg(0x02), c.reg(0x03)], [0x0D, 0x05]); // 32 MHz / 9600 = 3333
    assert_eq!([c.reg(0x04), c.reg(0x05)], [0x02, 0x3D]); // 35 kHz / 61.035 Hz = 573
    assert_eq!([c.reg(0x06), c.reg(0x07), c.reg(0x08)], [0xE7, 0x7F, 0x0A]); // 925.985 MHz
    assert_eq!(c.reg(0x09), 0x00); // PA minimum, RFO
    assert_eq!(c.reg(0x0C), 0x23); // LNA G1, HF boost
    assert_eq!(c.reg(0x0D), 0x1E); // AFC + AGC auto, trigger on preamble detect
    assert_eq!(c.reg(0x12), 0x02); // RxBw mantissa 16, exponent 2 = 125 kHz
    assert_eq!(c.reg(0x13), 0x02); // AfcBw the same
    assert_eq!(c.reg(0x1F), 0xAA); // preamble detector on, 2 bytes, tol 10
    assert_eq!(c.reg(0x27), 0xB1); // auto-restart w/ PLL, 0x55 preamble, sync on, 2 bytes
    assert_eq!([c.reg(0x28), c.reg(0x29)], [0x2D, 0xD4]); // sync word
    assert_eq!(c.reg(0x30), 0x80); // variable length, no DC-free, no hw CRC
    assert_eq!(c.reg(0x31), 0x40); // packet mode
    assert_eq!(c.reg(0x32), 64); // max payload
    assert_eq!(c.reg(0x35), 0x8F);
    assert_eq!(c.reg(0x40) >> 6, 0); // DIO0 = PayloadReady
    assert_eq!(c.reg(0x01) & 0xE8, 0x00); // FSK, HF band
}

#[test]
fn start_rx_reaches_rx_mode() {
    let (mut r, c) = radio();
    r.start_rx().unwrap();
    assert_eq!(c.reg(0x01) & 0x07, 5);
    assert!(matches!(r.mode().unwrap(), Mode::Rx));
}

#[test]
fn rejects_wrong_chip() {
    let chip = SimChip::new();
    chip.0.borrow_mut().mem[0x42] = 0x00;
    let mut r = Sx1276::new(chip, Pin);
    assert!(matches!(r.init(&mut NoDelay, &CFG), Err(sx1276::Error::BadVersion(0))));
}

#[test]
fn reads_a_packet_from_the_fifo() {
    let (mut r, c) = radio();
    assert!(!r.payload_ready().unwrap());
    c.0.borrow_mut().mem[0x3F] = 0x04; // PayloadReady
    c.0.borrow_mut().fifo.extend([5, b'$', b'G', b'P', b'G', b'G']);
    assert!(r.payload_ready().unwrap());
    let mut buf = [0u8; 64];
    let n = r.read_packet(&mut buf).unwrap();
    assert_eq!(&buf[..n], b"$GPGG");
    assert!(c.0.borrow().fifo.is_empty());
}

#[test]
fn oversized_packet_is_truncated_and_drained() {
    let (mut r, c) = radio();
    c.0.borrow_mut().fifo.extend([10u8].into_iter().chain(0..10));
    let mut buf = [0u8; 4];
    let n = r.read_packet(&mut buf).unwrap();
    assert_eq!(&buf[..n], &[0, 1, 2, 3]);
    assert!(c.0.borrow().fifo.is_empty());
}
