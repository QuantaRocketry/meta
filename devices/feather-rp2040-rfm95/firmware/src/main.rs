//! Adafruit Feather RP2040 RFM95: receive FSK packets and print them as ASCII
//! over USB CDC (a virtual serial port). Built on embassy.
//!
//! Hardware (from the Adafruit Feather RP2040 RFM95 pinout):
//!
//! | Signal       | GPIO | Function            |
//! |--------------|------|---------------------|
//! | SCK          | 14   | SPI1 SCK            |
//! | MOSI         | 15   | SPI1 TX             |
//! | MISO         | 8    | SPI1 RX             |
//! | RFM_CS       | 16   | chip select         |
//! | RFM_RST      | 17   | reset               |
//! | RFM_DIO0     | 21   | PayloadReady        |
//! | red LED (D13)| 13   | heartbeat           |
//!
//! Tasks: `radio_task` (init/retry, receive, status), `cdc_task` (feeds the USB
//! serial port from a byte pipe), `usb_task` (USB device), `heartbeat_task` (LED).
#![no_std]
#![no_main]

use core::fmt::Write as _;
use core::sync::atomic::{AtomicBool, Ordering};

use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::{SPI1, USB};
use embassy_rp::spi::{self, Blocking, Spi};
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::pipe::Pipe;
use embassy_time::{Delay, Duration, Timer};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::{Builder, Config, UsbDevice};
use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};
use heapless::String;
use panic_halt as _;
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

/// Centre frequency (Hz) and channel, as defined by eggtimer-decoder: the
/// signal is at `CENTER_FREQ_HZ + channel offset` (channel 0 = -15 kHz,
/// channel n >= 1 = 235 kHz + (n-1) * 250 kHz; only channel 0 has been measured).
const CENTER_FREQ_HZ: i32 = 925_000_000;
const CHANNEL: i32 = 4;
const CHANNEL_OFFSET_HZ: i32 = if CHANNEL == 0 {
    -15_000
} else {
    235_000 + (CHANNEL - 1) * 250_000
};

/// Radio settings: Eggtimer/Eggfinder TX (Si1000, 2-FSK, 9600 baud, +/-35 kHz,
/// preamble 0x55 x5, sync 2D D4, length byte, payload, CRC-16/XMODEM).
const RADIO: sx1276::Config = sx1276::Config {
    frequency_hz: (CENTER_FREQ_HZ + CHANNEL_OFFSET_HZ) as u32, // 925.985 MHz
    bitrate_bps: 9_600,
    fdev_hz: 35_000,
    // RxBw = 32 MHz / (16 * 2^(2+2)) = 125 kHz: covers 2*35 kHz + 9.6 kHz plus
    // crystal/tuning error on both ends.
    rx_bw: (sx1276::RxBwMantissa::M16, 2),
    sync_word: &[0x2D, 0xD4],
    preamble_is_0x55: true,
    // The TX appends CRC-16/XMODEM (init 0, over len+payload), which the
    // SX1276 hardware CRC (init 0x1D0F) cannot check, so it is off. The
    // trailing 2 CRC bytes are not read; non-ASCII packets are dropped instead.
    hw_crc: false,
    // AGC on, so it can adapt to the (strong) signal on the preamble. To try a
    // fixed gain instead set `agc_auto: false` and `lna_gain` 1 (max) to 6.
    agc_auto: true,
    lna_gain: 1,
};

/// How often the idle status line is printed.
const STATUS_PERIOD: Duration = Duration::from_secs(2);

type RadioDevice = ExclusiveDevice<Spi<'static, SPI1, Blocking>, Output<'static>, NoDelay>;
type Radio = sx1276::Sx1276<RadioDevice, Output<'static>>;
type UsbDriver = Driver<'static, USB>;

/// Bytes waiting to be sent to the host. Written by `radio_task`, drained by `cdc_task`.
static TX_PIPE: Pipe<CriticalSectionRawMutex, 2048> = Pipe::new();
/// Whether the radio is initialised; the LED blinks faster while it is not.
static RADIO_OK: AtomicBool = AtomicBool::new(false);

/// Packet counters: `raw` = read from the radio, `printed` = sent to USB,
/// `dropped` = rejected (non-ASCII/noise, or the pipe was full).
#[derive(Default)]
struct Stats {
    raw: u32,
    printed: u32,
    dropped: u32,
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // --- USB CDC serial. Set up first so a radio problem can never stop it
    // from enumerating. ----------------------------------------------------
    let driver = Driver::new(p.USB, Irqs);
    let mut config = Config::new(0x16c0, 0x27dd);
    config.manufacturer = Some("quanta");
    config.product = Some("Feather RFM95 FSK receiver");
    config.serial_number = Some("0001");
    config.max_power = 100;
    config.max_packet_size_0 = 64;
    // Required for Windows to treat the CDC ACM interface as a composite device.
    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config.composite_with_iads = true;

    static CONFIG_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static BOS_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();
    static CDC_STATE: StaticCell<State> = StaticCell::new();
    let mut builder = Builder::new(
        driver,
        config,
        CONFIG_DESC.init([0; 256]),
        BOS_DESC.init([0; 256]),
        &mut [], // no MS OS descriptors
        CONTROL_BUF.init([0; 64]),
    );
    let class = CdcAcmClass::new(&mut builder, CDC_STATE.init(State::new()), 64);
    let usb = builder.build();

    // --- Radio on SPI1 (blocking: transfers are a few bytes) ---------------
    let mut spi_config = spi::Config::default();
    spi_config.frequency = 4_000_000; // mode 0 is the default
    let spi = Spi::new_blocking(p.SPI1, p.PIN_14, p.PIN_15, p.PIN_8, spi_config);
    let cs = Output::new(p.PIN_16, Level::High);
    let rst = Output::new(p.PIN_17, Level::High);
    let dio0 = Input::new(p.PIN_21, Pull::Down);
    let radio: Radio = sx1276::Sx1276::new(ExclusiveDevice::new_no_delay(spi, cs), rst);

    let led = Output::new(p.PIN_13, Level::Low);

    spawner.spawn(usb_task(usb).unwrap());
    spawner.spawn(cdc_task(class).unwrap());
    spawner.spawn(heartbeat_task(led).unwrap());
    spawner.spawn(radio_task(radio, dio0).unwrap());
}

#[embassy_executor::task]
async fn usb_task(mut usb: UsbDevice<'static, UsbDriver>) -> ! {
    usb.run().await
}

/// Copy bytes from `TX_PIPE` to the serial port while a host has it open.
#[embassy_executor::task]
async fn cdc_task(mut class: CdcAcmClass<'static, UsbDriver>) -> ! {
    let max_packet = class.max_packet_size() as usize;
    let mut buf = [0u8; 64];
    loop {
        class.wait_connection().await;
        // Nobody was listening: do not replay stale backlog.
        TX_PIPE.clear();
        loop {
            let n = TX_PIPE.read(&mut buf).await;
            if class.write_packet(&buf[..n]).await.is_err() {
                break; // host went away
            }
            // A transfer that is an exact multiple of the packet size needs a
            // zero-length packet to tell the host it ended.
            if n == max_packet && class.write_packet(&[]).await.is_err() {
                break;
            }
        }
    }
}

/// Blink the red LED: 1 Hz = radio running, 5 Hz = radio not working.
#[embassy_executor::task]
async fn heartbeat_task(mut led: Output<'static>) -> ! {
    loop {
        led.toggle();
        let ms = if RADIO_OK.load(Ordering::Relaxed) { 500 } else { 200 };
        Timer::after_millis(ms).await;
    }
}

/// Bring the radio up (retrying forever, reporting failures over USB), then
/// receive packets, and print a status line whenever 2 s pass without one.
/// (A one-off "ready" message would be lost: it is sent before the host has
/// opened the port, so the status is repeated instead.)
#[embassy_executor::task]
async fn radio_task(mut radio: Radio, mut dio0: Input<'static>) -> ! {
    let mut stats = Stats::default();
    loop {
        let mut delay = Delay;
        if let Err(e) = radio.init(&mut delay, &RADIO).and_then(|_| radio.start_rx()) {
            RADIO_OK.store(false, Ordering::Relaxed);
            let mut line: String<96> = String::new();
            let _ = write!(line, "radio init failed: {:?}", e);
            enqueue_line(line.as_bytes());
            Timer::after_millis(500).await;
            continue;
        }
        RADIO_OK.store(true, Ordering::Relaxed);

        loop {
            // DIO0 stays high while a packet is pending, so drain until it is
            // low. Waiting for the *level* (not an edge) means a packet that
            // arrives mid-read is picked up on the next pass.
            while matches!(radio.payload_ready(), Ok(true)) {
                let mut buf = [0u8; sx1276::MAX_PAYLOAD];
                let Ok(n) = radio.read_packet(&mut buf) else { break };
                stats.raw += 1;
                if enqueue_payload(&buf[..n]) {
                    stats.printed += 1;
                } else {
                    stats.dropped += 1;
                }
            }

            // Keep the stream clean while packets flow (NMEA sentences span
            // packets): only report when nothing arrived for a while.
            if let Either::Second(()) =
                select(dio0.wait_for_high(), Timer::after(STATUS_PERIOD)).await
            {
                let mode = radio.mode().ok();
                let rssi = radio.rssi_dbm().unwrap_or(0);
                let khz = RADIO.frequency_hz / 1000;
                let mut line: String<128> = String::new();
                let _ = write!(
                    line,
                    "radio ok, mode={:?} ({}), {}.{:03} MHz ch{}, rssi={} dBm, packets raw={} printed={} dropped={}: waiting for packet",
                    mode,
                    if matches!(mode, Some(sx1276::Mode::Rx)) { "RX" } else { "NOT RX" },
                    khz / 1000,
                    khz % 1000,
                    CHANNEL,
                    rssi,
                    stats.raw,
                    stats.printed,
                    stats.dropped,
                );
                enqueue_line(line.as_bytes());
            }
        }
    }
}

/// Append a status/diagnostic line (`\r\n` terminated). Dropped whole if it does not fit.
fn enqueue_line(line: &[u8]) -> bool {
    if TX_PIPE.free_capacity() < line.len() + 2 {
        return false;
    }
    let _ = TX_PIPE.try_write(line);
    let _ = TX_PIPE.try_write(b"\r\n");
    true
}

/// Append a received payload verbatim. Eggtimer payloads are chunks (<= 32
/// bytes) of an NMEA text stream, so sentences span several packets and
/// carry their own line endings; adding any would break them up.
/// The CRC cannot be checked here, so a packet containing anything other
/// than printable ASCII / CR / LF (noise or bit errors) is dropped whole.
/// Returns whether it was queued.
fn enqueue_payload(payload: &[u8]) -> bool {
    let ascii = payload
        .iter()
        .all(|b| (0x20..=0x7E).contains(b) || matches!(b, b'\r' | b'\n'));
    if payload.is_empty() || !ascii || TX_PIPE.free_capacity() < payload.len() {
        return false;
    }
    TX_PIPE.try_write(payload).is_ok()
}
