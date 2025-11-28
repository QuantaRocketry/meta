#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use embassy_executor::Spawner;
use embassy_time::{Duration, Instant, Timer};
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::spi::{
    Mode,
    master::{Config, Spi},
};
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;

use embedded_graphics::{pixelcolor, prelude::*};
use embedded_hal_bus::spi::ExclusiveDevice;
use mipidsi::Builder;
use mipidsi::interface::SpiInterface;

use entangler_waveshare_esp32s3::display::JD9853;
use log::info;

type ColorSpace = pixelcolor::Rgb565;

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 73744);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0);

    info!("Embassy initialized!");

    // TODO: Spawn some tasks
    let _ = spawner;
    let mut delay = embassy_time::Delay {};

    let sclk = peripherals.GPIO38;
    let mosi = peripherals.GPIO39;
    let cs = Output::new(peripherals.GPIO21, Level::High, OutputConfig::default());
    let dc = Output::new(peripherals.GPIO45, Level::High, OutputConfig::default());
    let lcd_backlight = Output::new(peripherals.GPIO46, Level::High, OutputConfig::default());
    let rst = Output::new(peripherals.GPIO40, Level::High, OutputConfig::default());

    info!("Pins made!");

    let mut spi = Spi::new(
        peripherals.SPI2,
        Config::default()
            .with_frequency(Rate::from_mhz(40))
            .with_mode(Mode::_0),
    )
    .unwrap()
    .with_sck(sclk)
    .with_mosi(mosi);
    // .with_cs(cs);

    let spi_device = ExclusiveDevice::new(spi, cs, delay.clone()).unwrap();

    info!("Exclusive device made!");

    let mut buffer = [0_u8; 1024];

    // Create a DisplayInterface from SPI and DC pin, with no manual CS control
    let di = SpiInterface::new(spi_device, dc, &mut buffer);

    let mut display = Builder::new(JD9853, di)
        .reset_pin(rst)
        .color_order(mipidsi::options::ColorOrder::Bgr)
        .display_offset(34, 0)
        .display_size(172, 320)
        .init(&mut delay)
        .unwrap();

    let cols = [ColorSpace::RED, ColorSpace::GREEN, ColorSpace::BLUE];
    loop {
        for col in cols {
            let start = Instant::now();
            display.clear(col).expect("Failed to clear display");
            info!(
                "{} Cleared! Took {} sec",
                Instant::now().as_millis() as f32 / 1000.0,
                (Instant::now() - start).as_millis() as f32 / 1000.0
            );
            Timer::after(Duration::from_millis(500)).await;
        }
    }
}
