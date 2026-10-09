use crate::{
    device::hardware::Irqs, device::hardware::UsbResources, info, system::SystemState, warn,
};
use common::module::battery::BatteryState;
use core::fmt::Write;
use embassy_executor::Spawner;
use embassy_futures::{join, select};
use embassy_nrf::usb;
use embassy_sync::watch::DynReceiver;
use embassy_time::{Duration, Instant, Ticker, Timer};
use embassy_usb::{
    Builder, UsbDevice,
    class::cdc_acm::{self, CdcAcmClass},
    driver::{Driver, EndpointError},
};
use heapless::{String, format};
use qcp::{Message, MessageKind};
use static_cell::StaticCell;

type UsbDriver = usb::Driver<'static, usb::vbus_detect::HardwareVbusDetect>;

type BatteryWatcher = DynReceiver<'static, BatteryState>;

pub struct Disconnected {}

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected {},
        }
    }
}

pub async fn echo<'d>(class: &mut CdcAcmClass<'d, impl Driver<'d>>) -> Result<(), Disconnected> {
    let mut buf = [0; 64];
    loop {
        let n = class.read_packet(&mut buf).await?;
        let data = &buf[..n];
        info!("data: {:x}", data[0]);
        class.write_packet(data).await?;
    }
}

#[embassy_executor::task]
async fn echo_task(mut class: CdcAcmClass<'static, UsbDriver>) {
    loop {
        class.wait_connection().await;
        info!("Connected");
        let _ = echo(&mut class).await;
        info!("Disconnected");
    }
}

/// Write `data` to the host, splitting it into USB packets.
async fn write_all<'d, D: Driver<'d>>(
    class: &mut CdcAcmClass<'d, D>,
    data: &[u8],
) -> Result<(), Disconnected> {
    for chunk in data.chunks(class.max_packet_size() as usize) {
        class.write_packet(chunk).await?;
    }
    Ok(())
}

/// Build the reply for a decoded QCP message, if there is one.
fn qcp_reply(message: Message, battery: &mut BatteryWatcher) -> Option<Message> {
    match message {
        Message::Request(MessageKind::Uptime) => Some(Message::Uptime(Instant::now().as_micros())),
        Message::Request(MessageKind::Battery) => battery.try_get().map(Message::Battery),
        other => {
            info!("QCP: unhandled message {}", other_kind(&other));
            None
        }
    }
}

fn other_kind(message: &Message) -> &'static str {
    match message {
        Message::Request(_) => "Request",
        Message::Uptime(_) => "Uptime",
        Message::TrackedPOI(_) => "TrackedPOI",
        Message::RadioConfig(_) => "RadioConfig",
        Message::Protocol(_) => "Protocol",
        Message::Battery(_) => "Battery",
    }
}

/// Reassembles zero-terminated COBS frames from a byte stream and decodes
/// them into messages.
struct FrameDecoder {
    // Room for the largest message plus COBS overhead and the delimiter.
    buf: [u8; qcp::MESSAGE_SIZE_MAX + 4],
    len: usize,
    overflowed: bool,
}

impl FrameDecoder {
    const fn new() -> Self {
        Self {
            buf: [0; qcp::MESSAGE_SIZE_MAX + 4],
            len: 0,
            overflowed: false,
        }
    }

    /// Feed one byte, returning a message once a complete frame decodes.
    fn push(&mut self, byte: u8) -> Option<Message> {
        if byte != 0 {
            if self.len < self.buf.len() {
                self.buf[self.len] = byte;
                self.len += 1;
            } else {
                self.overflowed = true;
            }
            return None;
        }

        let len = core::mem::take(&mut self.len);
        if core::mem::take(&mut self.overflowed) || len == 0 {
            return None;
        }

        match postcard::from_bytes_cobs::<Message>(&mut self.buf[..len]) {
            Ok(message) => Some(message),
            Err(_) => {
                info!("QCP: failed to decode frame");
                None
            }
        }
    }
}

/// Encode `message` as a COBS frame and write it to the host.
async fn send_message<'d, D: Driver<'d>>(
    class: &mut CdcAcmClass<'d, D>,
    message: &Message,
) -> Result<(), Disconnected> {
    let mut out = [0u8; qcp::MESSAGE_SIZE_MAX + 4];
    match postcard::to_slice_cobs(message, &mut out) {
        Ok(encoded) => write_all(class, encoded).await,
        Err(_) => {
            info!("QCP: failed to encode message");
            Ok(())
        }
    }
}

/// Serve QCP over the CDC-ACM port. Requests from the host are answered as
/// they arrive, and the battery state is pushed whenever it changes, at most
/// once per second.
async fn qcp_serve<'d, D: Driver<'d>>(
    class: &mut CdcAcmClass<'d, D>,
    battery: &mut BatteryWatcher,
) -> Result<(), Disconnected> {
    let mut decoder = FrameDecoder::new();
    let mut battery_ticker = Ticker::every(Duration::from_hz(1));

    loop {
        let mut chunk = [0u8; 64];
        match select::select(
            class.read_packet(&mut chunk),
            join::join(battery.changed(), battery_ticker.next()),
        )
        .await
        {
            select::Either::First(n) => {
                for &b in &chunk[..n?] {
                    let Some(message) = decoder.push(b) else {
                        continue;
                    };
                    if let Some(reply) = qcp_reply(message, battery) {
                        send_message(class, &reply).await?;
                    }
                }
            }
            select::Either::Second((state, ())) => {
                send_message(class, &Message::Battery(state)).await?;
            }
        }
    }
}

#[embassy_executor::task]
async fn qcp_task(mut class: CdcAcmClass<'static, UsbDriver>, state: &'static SystemState) {
    info!("QCP task started");

    let Some(mut battery) = state.try_get_battery_state_watcher() else {
        warn!("QCP task ran out of battery watchers");
        return;
    };

    loop {
        class.wait_connection().await;
        let _ = qcp_serve(&mut class, &mut battery).await;
    }
}

#[embassy_executor::task]
async fn usb_task(mut device: UsbDevice<'static, UsbDriver>) {
    device.run().await;
}

#[embassy_executor::task]
async fn logger_task(class: CdcAcmClass<'static, UsbDriver>) {
    embassy_usb_logger::with_custom_style!(
        1024,
        log::LevelFilter::Debug,
        class,
        crate::log_style::log_style
    )
    .await;
}

#[embassy_executor::task]
pub async fn runner(spawner: Spawner, r: UsbResources, state: &'static SystemState) {
    let driver = usb::Driver::new(
        r.usbd,
        Irqs,
        usb::vbus_detect::HardwareVbusDetect::new(Irqs),
    );

    static SERIAL_STRING: StaticCell<heapless::string::String<17>> = StaticCell::new();
    let mac = crate::device::hardware::get_mac_addr();
    let bytes = mac.to_be_bytes(); // [u8; 8], big-endian

    let serial_string = SERIAL_STRING.init(
        heapless::format!(
            17;
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]
        )
        .unwrap(),
    );
    let mut config = embassy_usb::Config::new(0x2886, 0x1667);
    config.manufacturer = Some("Seeed");
    config.product = Some("Wio Tracker L1 Pro");
    config.serial_number = Some(serial_string.as_str());
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    static CONFIG_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static BOS_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static MSOS_DESC: StaticCell<[u8; 128]> = StaticCell::new();
    static CONTROL_BUF: StaticCell<[u8; 128]> = StaticCell::new();
    let mut builder = Builder::new(
        driver,
        config,
        &mut CONFIG_DESC.init([0; 256])[..],
        &mut BOS_DESC.init([0; 256])[..],
        &mut MSOS_DESC.init([0; 128])[..],
        &mut CONTROL_BUF.init([0; 128])[..],
    );

    // Create classes on the builder.
    static STATE: StaticCell<cdc_acm::State> = StaticCell::new();
    let cdc_state = STATE.init(cdc_acm::State::new());
    let class = CdcAcmClass::new(&mut builder, cdc_state, 64);

    // Start the log hook if not defmt
    #[cfg(not(feature = "defmt"))]
    {
        static LOG_STATE: StaticCell<cdc_acm::State> = StaticCell::new();
        let log_state = LOG_STATE.init(cdc_acm::State::new());
        let logging_class = CdcAcmClass::new(&mut builder, log_state, 64);
        spawner.spawn(logger_task(logging_class).unwrap());
    }

    spawner.spawn(qcp_task(class, state).unwrap());
    // spawner.spawn(echo_task(class).unwrap());

    // Build the builder.
    let usb = builder.build();
    spawner.spawn(usb_task(usb).unwrap());
}
