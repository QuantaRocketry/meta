use crate::{device::hardware::Irqs, device::hardware::UsbResources, info, terminal::AtCommand};
use at_commands::parser::CommandParser;
use core::fmt::Write;
use embassy_executor::Spawner;
use embassy_nrf::usb;
use embassy_usb::{
    Builder, UsbDevice,
    class::cdc_acm::{self, CdcAcmClass},
    driver::{Driver, EndpointError},
};
use heapless::{String, format};
use static_cell::StaticCell;

type UsbDriver = usb::Driver<'static, usb::vbus_detect::HardwareVbusDetect>;

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

fn format_freq(freq_hz: u32) -> String<32> {
    let mut s = String::new();
    let _ = write!(s, "+FREQ: {}\r\n", freq_hz);
    s
}

async fn handle_command<'d, D: embassy_usb::driver::Driver<'d>>(
    class: &mut CdcAcmClass<'d, D>,
    line: &[u8],
) {
    if CommandParser::parse(line)
        .expect_identifier(b"AT?")
        .finish()
        .is_ok()
    {
        info!("CLI: AT?");

        let _ = class.write_packet(b"+OK\r\n").await;
        return;
    }

    if CommandParser::parse(line)
        .expect_identifier(b"AT+FREQ?")
        .finish()
        .is_ok()
    {
        info!("CLI: AT+FREQ?");

        let resp = format_freq(0);
        let _ = class.write_packet(resp.as_bytes()).await;
        return;
    }

    if let Ok((hz,)) = CommandParser::parse(line)
        .expect_identifier(b"AT+FREQ=")
        .expect_int_parameter()
        .finish()
    {
        info!("CLI: AT+FREQ={}", &hz);

        // radio.set_freq_hz(hz as u32); // your radio driver call
        let _ = class.write_packet(b"+OK\r\n").await;
        return;
    }

    let _ = class.write_packet(b"ERROR\r\n").await;
}

#[embassy_executor::task]
async fn cli_task(mut class: CdcAcmClass<'static, UsbDriver>) {
    info!("CLI task started");

    let mut cmd_buf = [0u8; 64];
    let mut cmd_len = 0usize;

    loop {
        class.wait_connection().await;
        loop {
            let mut chunk = [0u8; 64];
            let n = match class.read_packet(&mut chunk).await {
                Ok(n) => n,
                Err(_) => break,
            };
            for &b in &chunk[..n] {
                if b == b'\n' {
                    if let Some(cmd) = crate::terminal::handle_command(&cmd_buf[..cmd_len]) {
                        match cmd {
                            AtCommand::Ping => {
                                let _ = class.write_packet(b"+OK\r\n").await;
                            }
                            AtCommand::GetFrequency => {
                                let _ = class.write_packet(b"+Frequency=0\r\n").await;
                            }
                            AtCommand::SetFrequency(_hz) => {
                                let _ = class.write_packet(b"+OK\r\n").await;
                            }
                            AtCommand::GetSF => {
                                let _ = class.write_packet(b"+Frequency=0\r\n").await;
                            }
                            AtCommand::SetSF(_) => {
                                let _ = class.write_packet(b"+OK\r\n").await;
                            }
                        }
                    } else {
                        info!("NONE");
                    };
                    cmd_len = 0;
                } else if b != b'\r' && cmd_len < cmd_buf.len() {
                    cmd_buf[cmd_len] = b;
                    cmd_len += 1;
                }
            }
        }
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
pub async fn runner(spawner: Spawner, r: UsbResources) {
    let driver = usb::Driver::new(
        r.usbd,
        Irqs,
        usb::vbus_detect::HardwareVbusDetect::new(Irqs),
    );

    static SERIAL_STRING: StaticCell<heapless::string::String<17>> = StaticCell::new();
    let mac = crate::get_mac_addr();
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
    let state = STATE.init(cdc_acm::State::new());
    let class = CdcAcmClass::new(&mut builder, state, 64);

    // Start the log hook if not defmt
    #[cfg(not(feature = "defmt"))]
    {
        static LOG_STATE: StaticCell<cdc_acm::State> = StaticCell::new();
        let log_state = LOG_STATE.init(cdc_acm::State::new());
        let logging_class = CdcAcmClass::new(&mut builder, log_state, 64);
        spawner.spawn(logger_task(logging_class).unwrap());
    }

    spawner.spawn(cli_task(class).unwrap());
    // spawner.spawn(echo_task(class).unwrap());

    // Build the builder.
    let usb = builder.build();
    spawner.spawn(usb_task(usb).unwrap());
}
