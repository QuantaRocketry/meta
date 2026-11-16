//! Adafruit Feather RP2040 RFM95: receive (G)FSK packets and pass them to the
//! host over USB CDC as qcp `RawPayload` messages. The host chooses what to
//! listen for by sending a qcp `Protocol` or `RadioConfig`; the feather retunes
//! and acknowledges with the `RadioConfig` it applied, or answers `Error`.
//! Built on embassy.
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
//! | NeoPixel     | 4    | packet / config     |
//!
//! Tasks: `radio_task` (init/retry, receive, apply configs), `cdc_task` (qcp
//! over the USB serial port), `usb_task` (USB device), `heartbeat_task` (LED), `neopixel_task` (status pixel).
#![no_std]
#![no_main]

mod neopixel;

use core::cell::Cell;
use core::sync::atomic::{AtomicBool, Ordering};

use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::{DMA_CH0, PIO0, SPI1, USB};
use embassy_rp::pio::{self, Pio};
use embassy_rp::pio_programs::ws2812::{PioWs2812, PioWs2812Program};
use embassy_rp::spi::{self, Blocking, Spi};
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::signal::Signal;
use embassy_time::{Delay, Instant, Timer};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::{Builder, Config, UsbDevice};
use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};
use panic_halt as _;
use qcp::{ConfigError, FRAME_SIZE_MAX, FrameDecoder, Message, MessageKind, RadioConfig, RawPayload};
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
    PIO0_IRQ_0 => pio::InterruptHandler<PIO0>;
    DMA_IRQ_0 => embassy_rp::dma::InterruptHandler<DMA_CH0>;
});

type RadioDevice = ExclusiveDevice<Spi<'static, SPI1, Blocking>, Output<'static>, NoDelay>;
type Radio = sx1276::Sx1276<RadioDevice, Output<'static>>;
type UsbDriver = Driver<'static, USB>;

/// What the host asked the radio to receive: a protocol, or a raw waveform.
/// The feather acknowledges with the same kind it was sent.
#[derive(Clone, Copy)]
enum Selection {
    Protocol(qcp::Protocol),
    Radio(RadioConfig),
}

impl Selection {
    /// The waveform to tune to, or `None` for a protocol with none defined.
    fn radio(self) -> Option<RadioConfig> {
        match self {
            Selection::Protocol(protocol) => protocol.radio_config(),
            Selection::Radio(config) => Some(config),
        }
    }

    fn message(self) -> Message {
        match self {
            Selection::Protocol(protocol) => Message::Protocol(protocol),
            Selection::Radio(config) => Message::RadioConfig(config),
        }
    }
}

/// A selection from the host, already checked to be receivable. Written by
/// `cdc_task`, taken by `radio_task`.
static NEW_CONFIG: Signal<CriticalSectionRawMutex, Selection> = Signal::new();
/// Messages for the host. Written by `radio_task`, drained by `cdc_task`.
/// Writers never wait: a message is dropped if the host is not keeping up.
static TO_HOST: Channel<CriticalSectionRawMutex, Message, 4> = Channel::new();
/// What the radio is running, for `Request(Protocol)` and `Request(RadioConfig)`.
static CURRENT: Mutex<CriticalSectionRawMutex, Cell<Option<Selection>>> =
    Mutex::new(Cell::new(None));
/// Whether the radio is initialised; the LED blinks faster while it is not.
static RADIO_OK: AtomicBool = AtomicBool::new(false);

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // --- USB CDC serial. Set up first so a radio problem can never stop it
    // from enumerating. ----------------------------------------------------
    let driver = Driver::new(p.USB, Irqs);
    let mut config = Config::new(0x16c0, 0x27dd);
    config.manufacturer = Some("quanta");
    config.product = Some("Feather RFM95 qcp receiver");
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

    let Pio { mut common, sm0, .. } = Pio::new(p.PIO0, Irqs);
    let program = PioWs2812Program::new(&mut common);
    let pixel = PioWs2812::new(&mut common, sm0, p.DMA_CH0, Irqs, p.PIN_4, &program);

    spawner.spawn(usb_task(usb).unwrap());
    spawner.spawn(cdc_task(class).unwrap());
    spawner.spawn(heartbeat_task(led).unwrap());
    spawner.spawn(neopixel::neopixel_task(pixel).unwrap());
    spawner.spawn(radio_task(radio, dio0).unwrap());
}

#[embassy_executor::task]
async fn usb_task(mut usb: UsbDevice<'static, UsbDriver>) -> ! {
    usb.run().await
}

/// What a message from the host asks of the feather.
enum Reply {
    /// Send this to the host now.
    Now(Message),
    /// Nothing to say yet: `radio_task` answers once the config is applied.
    Later,
    None,
}

/// Act on one message from the host. Configs are checked here, so a config
/// the radio cannot receive is refused before it disturbs the radio.
fn handle(message: Message) -> Reply {
    let selection = match message {
        Message::Request(MessageKind::Uptime) => {
            return Reply::Now(Message::Uptime(Instant::now().as_micros()));
        }
        Message::Request(MessageKind::Protocol) => {
            return match CURRENT.lock(Cell::get) {
                Some(selection @ Selection::Protocol(_)) => Reply::Now(selection.message()),
                _ => Reply::None,
            };
        }
        Message::Request(MessageKind::RadioConfig) => {
            return match CURRENT.lock(Cell::get).and_then(Selection::radio) {
                Some(config) => Reply::Now(Message::RadioConfig(config)),
                None => Reply::None,
            };
        }
        Message::RadioConfig(config) => Selection::Radio(config),
        Message::Protocol(protocol) => Selection::Protocol(protocol),
        _ => return Reply::None,
    };
    let checked = selection
        .radio()
        .ok_or(ConfigError::UnsupportedProtocol)
        .and_then(|config| sx1276::Config::try_from(&config).map(|_| ()));
    match checked {
        Ok(()) => {
            NEW_CONFIG.signal(selection);
            Reply::Later
        }
        Err(e) => Reply::Now(Message::Error(e)),
    }
}

/// Send `message` to the host as one frame, split into USB packets.
async fn send(class: &mut CdcAcmClass<'static, UsbDriver>, message: &Message) -> Result<(), ()> {
    let mut buf = [0u8; FRAME_SIZE_MAX];
    let Ok(frame) = qcp::encode_frame(message, &mut buf) else { return Ok(()) };
    let max_packet = class.max_packet_size() as usize;
    for chunk in frame.chunks(max_packet) {
        class.write_packet(chunk).await.map_err(|_| ())?;
    }
    // A transfer that is an exact multiple of the packet size needs a
    // zero-length packet to tell the host it ended.
    if frame.len() % max_packet == 0 {
        class.write_packet(&[]).await.map_err(|_| ())?;
    }
    Ok(())
}

/// qcp over the serial port while a host has it open.
#[embassy_executor::task]
async fn cdc_task(mut class: CdcAcmClass<'static, UsbDriver>) -> ! {
    let mut decoder = FrameDecoder::new();
    loop {
        class.wait_connection().await;
        // Nobody was listening: do not replay stale backlog.
        TO_HOST.clear();
        let _ = serve(&mut class, &mut decoder).await;
    }
}

async fn serve(
    class: &mut CdcAcmClass<'static, UsbDriver>,
    decoder: &mut FrameDecoder,
) -> Result<(), ()> {
    let mut chunk = [0u8; 64];
    loop {
        match select(class.read_packet(&mut chunk), TO_HOST.receive()).await {
            Either::First(n) => {
                for &byte in &chunk[..n.map_err(|_| ())?] {
                    let Some(message) = decoder.push(byte) else { continue };
                    if let Reply::Now(reply) = handle(message) {
                        send(class, &reply).await?;
                    }
                }
            }
            Either::Second(message) => send(class, &message).await?,
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

/// Bring the radio up on the default eggtimer config (retrying forever), then
/// receive packets. A config from the host replaces the running one.
#[embassy_executor::task]
async fn radio_task(mut radio: Radio, mut dio0: Input<'static>) -> ! {
    let mut selection = Selection::Protocol(qcp::Protocol::Eggtimer(Default::default()));
    let mut reported_fault = false;
    let mut from_host = false;
    loop {
        let config = selection.radio();
        let applied = config
            .as_ref()
            .ok_or(())
            .and_then(|c| sx1276::Config::try_from(c).map_err(|_| ()))
            .and_then(|cfg| {
                radio.init(&mut Delay, &cfg).and_then(|_| radio.start_rx()).map_err(|_| ())
            });
        if applied.is_err() {
            RADIO_OK.store(false, Ordering::Relaxed);
            if !core::mem::replace(&mut reported_fault, true) {
                let _ = TO_HOST.try_send(Message::Error(ConfigError::RadioFault));
            }
            // A newer config may be the fix; otherwise try this one again.
            if let Either::First(new) =
                select(NEW_CONFIG.wait(), Timer::after_millis(500)).await
            {
                selection = new;
                from_host = true;
                reported_fault = false;
            }
            continue;
        }
        RADIO_OK.store(true, Ordering::Relaxed);
        reported_fault = false;
        CURRENT.lock(|c| c.set(Some(selection)));
        let _ = TO_HOST.try_send(selection.message());
        if core::mem::take(&mut from_host) {
            neopixel::notify(neopixel::Notify::ConfigApplied);
        }

        // Without a chip-checked CRC, noise and bit errors get through, so
        // keep only packets that look like the ASCII (NMEA) eggtimer sends.
        let ascii_only = !matches!(config.as_ref().map(sx1276::Config::try_from), Some(Ok(c)) if c.hw_crc);

        loop {
            // DIO0 stays high while a packet is pending, so drain until it is
            // low. Waiting for the *level* (not an edge) means a packet that
            // arrives mid-read is picked up on the next pass.
            while matches!(radio.payload_ready(), Ok(true)) {
                let mut buf = [0u8; sx1276::MAX_PAYLOAD];
                let Ok(n) = radio.read_packet(&mut buf) else { break };
                neopixel::notify(neopixel::Notify::PacketReceived);
                forward_payload(&buf[..n], ascii_only);
            }

            match select(dio0.wait_for_high(), NEW_CONFIG.wait()).await {
                Either::First(()) => {}
                Either::Second(new) => {
                    selection = new;
                    from_host = true;
                    break;
                }
            }
        }
    }
}

/// Queue a received payload for the host verbatim: eggtimer payloads are
/// chunks (<= 32 bytes) of an NMEA text stream, so the host reassembles
/// sentences. Dropped if empty, not ASCII when `ascii_only`, or if the host
/// is not keeping up.
fn forward_payload(payload: &[u8], ascii_only: bool) {
    let ascii = payload
        .iter()
        .all(|b| (0x20..=0x7E).contains(b) || matches!(b, b'\r' | b'\n'));
    if payload.is_empty() || (ascii_only && !ascii) {
        return;
    }
    if let Ok(payload) = RawPayload::from_slice(payload) {
        let _ = TO_HOST.try_send(Message::RawPayload(payload));
    }
}
