#![no_std]
#![no_main]
#![allow(unused_imports)]
#![allow(unused_variables)]

extern crate alloc;
#[cfg(not(feature = "defmt"))]
use core::panic::PanicInfo;

use alloc::string::String;

use common::config::ConfigItem;
use common::module::battery::BatteryState;
use embassy_nrf::pac::radio;
use embassy_sync::lazy_lock::LazyLock;
use embassy_sync::watch;
use embassy_sync::zerocopy_channel::{self, Receiver};

#[cfg(feature = "defmt")]
pub use defmt::{debug, error, info, warn};
#[cfg(not(feature = "defmt"))]
pub use log::{debug, error, info, warn};

#[cfg(feature = "defmt")]
use defmt::{panic, unwrap};
#[cfg(feature = "defmt")]
use defmt_rtt as _;

use embassy_executor::Spawner;
use embassy_nrf::gpio::{self};
use embassy_nrf::{Peri, bind_interrupts, buffered_uarte, pac, peripherals, saadc, usb};
use embassy_sync::blocking_mutex::raw::{NoopRawMutex, ThreadModeRawMutex};
use embassy_time::{Duration, Instant, Timer, with_timeout};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::{Builder, UsbDevice};
use oled_async::displayrotation::DisplayRotation::{self};
use static_cell::{ConstStaticCell, StaticCell};

mod device;
mod log_style;
mod module;
mod system;

#[cfg(feature = "defmt")]
use panic_probe as _;
#[cfg(not(feature = "defmt"))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // Reset the chip
    cortex_m::peripheral::SCB::sys_reset();
}

use device::hardware::*;

use embedded_alloc::LlffHeap as Heap;
#[global_allocator]
static HEAP: Heap = Heap::empty();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let r = split_resources!(p);

    unsafe {
        embedded_alloc::init!(HEAP, 1024);
    }

    pac::CLOCK.tasks_hfclkstart().write_value(1);
    while pac::CLOCK.events_hfclkstarted().read() != 1 {}

    static SYSTEM_CONFIG: LazyLock<system::SystemConfig> =
        LazyLock::new(|| system::SystemConfig::new());

    static SYSTEM_STATE: LazyLock<system::SystemState> =
        LazyLock::new(|| system::SystemState::new(SYSTEM_CONFIG.get()));

    static RADIO_TX_BUFFERS: StaticCell<
        [common::module::radio::RadioBuffer; common::module::radio::RADIO_TX_CHANNEL_LENGTH],
    > = StaticCell::new();
    let radio_tx_buffers = RADIO_TX_BUFFERS.init(
        [const { common::module::radio::RadioBuffer::new() };
            common::module::radio::RADIO_TX_CHANNEL_LENGTH],
    );

    static RADIO_TX_CHANNEL: StaticCell<
        zerocopy_channel::Channel<'static, system::Mutex, common::module::radio::RadioBuffer>,
    > = StaticCell::new();
    let radio_tx_channel = RADIO_TX_CHANNEL.init(zerocopy_channel::Channel::new(radio_tx_buffers));
    let (radio_sender, radio_receiver) = radio_tx_channel.split();

    // Load stored config before any task starts reading it.
    let flash = device::build_config_flash(r.flash).await;
    SYSTEM_CONFIG
        .get()
        .init_storage(system::ConfigStorage::new(flash))
        .await;

    spawner.spawn(module::usb::runner(spawner, r.usb, SYSTEM_STATE.get()).unwrap());
    spawner.spawn(module::blink::runner(spawner, r.led).unwrap());
    // spawner.spawn(module::gnss::runner(spawner, r.gnss, SYSTEM_STATE.get()).unwrap());
    spawner.spawn(battery_runner(r.battery, SYSTEM_STATE.get().battery_sender()).unwrap());
    spawner.spawn(radio_runner(r.radio, SYSTEM_STATE.get(), radio_receiver).unwrap());
    // spawner.spawn(module::buzzer::runner(spawner, r.buzzer).unwrap());
    spawner.spawn(module::oled::runner(spawner, r.oled, SYSTEM_STATE.get()).unwrap());
    spawner.spawn(module::joystick::runner(spawner, r.joystick).unwrap());
    spawner.spawn(module::compass::runner(spawner, r.compass).unwrap());
    spawner.spawn(system::runner(spawner, SYSTEM_STATE.get(), radio_sender).unwrap());
}

#[embassy_executor::task]
async fn radio_runner(
    r: crate::device::hardware::RadioResources,
    system_state: &'static system::SystemState,
    receiver: Receiver<'static, system::Mutex, common::module::radio::RadioBuffer>,
) {
    let (spi, dio) = device::build_radio_resources(r);
    let settings = system_state.config.try_get_radio_config_manager().unwrap();
    info!("starting radio....");
    common::module::radio::runner(settings, spi, dio, receiver).await;
}

#[embassy_executor::task]
async fn battery_runner(
    r: crate::device::hardware::BatteryResources,
    sender: watch::DynSender<'static, BatteryState>,
) {
    let battery = crate::device::hardware::BatteryHardware::from_resources(r).await;
    common::module::battery::runner(battery, sender).await;
}
