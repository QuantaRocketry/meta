use alloc::format;
use embassy_executor::Spawner;
use embassy_nrf::{buffered_uarte, gpio, uarte};
use embassy_time::{Duration, Timer, with_timeout};
use nmea_stream::{self, NmeaReader, nmea::ParseResult};

use crate::{debug, device::hardware::GnssResources, device::hardware::Irqs, error, info};

#[derive(Default, Debug, Copy, Clone)]
pub struct GnssState {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f32,
}

pub trait GnssStateManager<'a> {
    fn set_gnss_state(&self, config: &GnssState);
    fn try_get_gnss_state_watcher(
        &'a self,
    ) -> Option<embassy_sync::watch::DynReceiver<'a, GnssState>>;
}

#[embassy_executor::task]
pub async fn runner(
    spawner: Spawner,
    mut r: GnssResources,
    state: &'static crate::system::SystemState,
) {
    run_gnss(spawner, r, state).await;
}

pub async fn run_gnss(
    _spawner: Spawner,
    mut r: GnssResources,
    state: &'_ impl GnssStateManager<'_>,
) {
    info!("GNSS task started");

    // enable gnss
    let mut gpio_rst = gpio::Output::new(r.rst_pin, gpio::Level::Low, gpio::OutputDrive::Standard);
    gpio_rst.set_high();
    Timer::after_millis(100).await;
    gpio_rst.set_low();

    // trigger wakeup
    let mut gpio_wakeup =
        gpio::Output::new(r.wakeup_pin, gpio::Level::High, gpio::OutputDrive::Standard);
    gpio_wakeup.set_low();
    Timer::after_millis(100).await;
    gpio_wakeup.set_high();

    Timer::after_secs(3).await;

    let baud_rates = [
        uarte::Baudrate::BAUD9600,
        uarte::Baudrate::BAUD115200,
        uarte::Baudrate::BAUD38400,
        uarte::Baudrate::BAUD57600,
    ];

    let mut final_baudrate = uarte::Baudrate::BAUD9600; // Default fallback
    info!("Searching for GNSS baud rate...");

    for baud in baud_rates {
        debug!("{}", format!("attempting baudrate: {:?}", baud).as_str());

        let mut config = uarte::Config::default();
        config.parity = uarte::Parity::EXCLUDED;
        config.baudrate = baud;

        let mut tx_buffer = [0u8; 256];
        let mut rx_buffer = [0u8; 256];

        // Mutably borrow the peripherals so we don't consume them during the search
        let mut uart = buffered_uarte::BufferedUarte::new(
            r.uart.reborrow(),
            r.timer.reborrow(),
            r.ppi0.reborrow(),
            r.ppi1.reborrow(),
            r.ppi_group.reborrow(),
            r.rx_pin.reborrow(),
            r.tx_pin.reborrow(),
            Irqs,
            config,
            &mut rx_buffer,
            &mut tx_buffer,
        );

        let mut buf = [0; 16];

        // Wait up to 1.5s (GNSS usually outputs at 1Hz, so 1.5s guarantees we catch a burst)
        match with_timeout(Duration::from_millis(5000), uart.read(&mut buf)).await {
            Ok(Ok(bytes_read)) => {
                // Check if the buffer contains the standard NMEA '$' character
                if buf[0..bytes_read].contains(&b'$') {
                    info!(
                        "Found valid GNSS data at {}!",
                        format!("{:?}", baud).as_str()
                    );
                    final_baudrate = baud;
                    break;
                }
            }
            Ok(Err(e)) => {
                error!("{:?}", e);
            }
            _ => {
                // Read error, framing error, or timeout — loop continues to the next baud rate.
                // The `uart` instance is dropped here, safely stopping the UARTE DMA.
            }
        }
    }

    // Initialize the final UART instance using the discovered baud rate.
    // We pass the peripherals by value here to consume them for the infinite loop.
    let mut config = uarte::Config::default();
    config.parity = uarte::Parity::EXCLUDED;
    config.baudrate = final_baudrate;

    let mut tx_buffer = [0u8; 4096];
    let mut rx_buffer = [0u8; 4096];
    let mut uart = buffered_uarte::BufferedUarte::new(
        r.uart.reborrow(),
        r.timer.reborrow(),
        r.ppi0.reborrow(),
        r.ppi1.reborrow(),
        r.ppi_group.reborrow(),
        r.rx_pin.reborrow(),
        r.tx_pin.reborrow(),
        Irqs,
        config,
        &mut rx_buffer,
        &mut tx_buffer,
    );

    let mut stream = NmeaReader::new();
    let mut gnss_state = GnssState::default();
    loop {
        if let Ok(sentence) = stream.next(&mut uart).await {
            debug!("{}", format!("{:?}", &sentence).as_str());
            match &sentence {
                ParseResult::GGA(gga_data) => {
                    if let Some(lat) = gga_data.latitude {
                        // gnss_state.latitude = 1.0;
                        gnss_state.latitude = lat;
                    }
                    if let Some(lng) = gga_data.longitude {
                        gnss_state.longitude = lng;
                    }
                    if let Some(alt) = gga_data.altitude {
                        gnss_state.altitude = alt;
                    }
                }
                ParseResult::RMC(rmc_data) => {
                    if let Some(lat) = rmc_data.lat {
                        gnss_state.latitude = lat;
                    }
                    if let Some(lng) = rmc_data.lon {
                        gnss_state.longitude = lng;
                    }
                }
                _ => (),
            };

            state.set_gnss_state(&gnss_state);
        };
    }
}
