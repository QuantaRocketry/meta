#![no_std]
#![no_main]
#![allow(unused_imports)]
#![allow(unused_variables)]

extern crate alloc;
#[cfg(not(feature = "defmt"))]
use core::panic::PanicInfo;

use alloc::string::String;

#[cfg(feature = "defmt")]
pub use defmt::{debug, error, info, warn};
use embassy_nrf::pac::radio;
use embassy_sync::lazy_lock::LazyLock;
use embassy_sync::zerocopy_channel;
#[cfg(not(feature = "defmt"))]
pub use log::{debug, error, info, warn};

#[cfg(feature = "defmt")]
use defmt::{panic, unwrap};

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

#[cfg(feature = "defmt")]
use panic_probe as _;

mod device;
mod log_style;
mod module;
mod system;
mod terminal;

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

pub fn get_mac_addr() -> u64 {
    let addr0 = pac::FICR.deviceaddr(0).read() as u64;
    let addr1 = pac::FICR.deviceaddr(1).read() as u64;

    let mac_48 = (addr0 | (addr1 << 32)) | 0xC000_0000_0000;
    return mac_48;
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let r = split_resources!(p);

    unsafe {
        embedded_alloc::init!(HEAP, 1024);
    }

    info!("Enabling ext hfosc...");
    pac::CLOCK.tasks_hfclkstart().write_value(1);
    while pac::CLOCK.events_hfclkstarted().read() != 1 {}

    static SYSTEM_CONFIG: LazyLock<system::SystemConfig> =
        LazyLock::new(|| system::SystemConfig::default());

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
        zerocopy_channel::Channel<'static, NoopRawMutex, common::module::radio::RadioBuffer>,
    > = StaticCell::new();
    let radio_tx_channel = RADIO_TX_CHANNEL.init(zerocopy_channel::Channel::new(radio_tx_buffers));
    let (radio_sender, radio_receiver) = radio_tx_channel.split();

    spawner.spawn(module::usb::runner(spawner, r.usb).unwrap());
    spawner.spawn(module::blink::runner(spawner, r.led).unwrap());
    // spawner.spawn(module::gnss::runner(spawner, r.gnss, SYSTEM_STATE.get()).unwrap());
    spawner.spawn(module::battery::runner(spawner, r.battery, SYSTEM_STATE.get()).unwrap());
    spawner.spawn(radio_runner(r.radio, SYSTEM_STATE.get()).unwrap());

    #[cfg(feature = "has-buzzer")]
    spawner.spawn(module::buzzer::runner(spawner, r.buzzer).unwrap());

    #[cfg(feature = "has-oled")]
    spawner.spawn(module::oled::runner(spawner, r.oled, SYSTEM_STATE.get()).unwrap());

    #[cfg(feature = "has-joystick")]
    spawner.spawn(module::joystick::runner(spawner, r.joystick).unwrap());

    #[cfg(feature = "has-compass")]
    spawner.spawn(module::compass::runner(spawner, r.compass).unwrap());

    spawner.spawn(system::runner(spawner, SYSTEM_STATE.get(), radio_sender).unwrap());
}

#[embassy_executor::task]
async fn radio_runner(
    r: crate::device::hardware::RadioResources,
    system_state: &'static system::SystemState,
) {
    let res = device::build_radio_resources(r);
    common::module::radio::runner(system_state, system_state).await;
}
