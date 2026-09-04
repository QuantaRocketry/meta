use embassy_nrf::{gpio, spim};

#[cfg(feature = "wio-l1")]
pub mod wio_l1;
use embedded_hal_bus::spi::ExclusiveDevice;
#[cfg(feature = "wio-l1")]
pub use wio_l1 as hardware;

#[cfg(feature = "solar-p1")]
pub mod solar_p1;
#[cfg(feature = "solar-p1")]
pub use solar_p1 as hardware;

use crate::device::hardware::RadioResources;

pub fn build_radio_resources(
    r: RadioResources,
) -> (
    ExclusiveDevice<spim::Spim<'static>, gpio::Output<'static>, embassy_time::Delay>,
    gpio::Input<'static>,
) {
    let mut config = spim::Config::default();
    config.frequency = spim::Frequency::M4; // Set clock speed
    config.mode = spim::MODE_0; // CPOL=0, CPHA=0
    config.orc = 0x00; // Over-read character

    let nss = gpio::Output::new(r.cs, gpio::Level::High, gpio::OutputDrive::Standard);
    let reset = gpio::Output::new(r.reset, gpio::Level::High, gpio::OutputDrive::Standard);
    let dio1 = gpio::Input::new(r.dio1, gpio::Pull::Down);
    let busy = gpio::Input::new(r.busy, gpio::Pull::None);
    let rf_switch_rx = gpio::Output::new(r.sw, gpio::Level::Low, gpio::OutputDrive::Standard);

    let spim = spim::Spim::new(r.spi, hardware::Irqs, r.sck, r.miso, r.mosi, config);
    let spi_device = ExclusiveDevice::new(spim, nss, embassy_time::Delay);

    return (spi_device, dio1);
}
