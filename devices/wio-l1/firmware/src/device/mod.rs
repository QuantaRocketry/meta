use core::ops::Range;

use embassy_nrf::{gpio, qspi, spim};
use embedded_hal_bus::spi::ExclusiveDevice;
pub mod hardware;

use crate::device::hardware::{BatteryResources, FlashResources, RadioResources};
use crate::{info, warn};

/// External P25Q16SH: 2MB, 4KB sectors.
pub type ConfigFlash = qspi::Qspi<'static>;
const FLASH_CAPACITY: u32 = 2 * 1024 * 1024;
const JEDEC_ID_P25Q16SH: [u8; 3] = [0x85, 0x60, 0x15];

/// Region of the external flash reserved for config storage (4 sectors).
pub const CONFIG_FLASH_RANGE: Range<u32> = 0x0000..0x4000;

pub async fn build_config_flash(r: FlashResources) -> ConfigFlash {
    let mut config = qspi::Config::default();
    config.capacity = FLASH_CAPACITY;
    config.frequency = qspi::Frequency::M32;
    // Single-line IO so we don't have to set the flash's quad-enable bit.
    // Config data is tiny, so throughput doesn't matter.
    config.read_opcode = qspi::ReadOpcode::FASTREAD;
    config.write_opcode = qspi::WriteOpcode::PP;

    let mut flash = qspi::Qspi::new(
        r.qspi,
        hardware::Irqs,
        r.sck,
        r.csn,
        r.io0,
        r.io1,
        r.io2,
        r.io3,
        config,
    );

    // Release from deep power-down in case the bootloader left it asleep.
    if let Err(e) = flash.custom_instruction(0xAB, &[], &mut []).await {
        warn!("flash: release power-down failed: {:?}", e);
    }

    // Verify chip ID
    let mut id = [0u8; 3];
    match flash.custom_instruction(0x9F, &[], &mut id).await {
        Ok(()) if id == JEDEC_ID_P25Q16SH => info!("flash: P25Q16SH detected"),
        Ok(()) => warn!(
            "flash: unexpected JEDEC id {:x} {:x} {:x}",
            id[0], id[1], id[2]
        ),
        Err(e) => warn!("flash: JEDEC id read failed: {:?}", e),
    }

    flash
}

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
