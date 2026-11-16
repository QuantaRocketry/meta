#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::{
    gpio::{self, AnyPin, Flex, OutputOpenDrain},
    peripherals::USB,
    usb::Driver,
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Channel};
use embassy_time::Timer;
use embassy_usb::{
    Config, UsbDevice,
    class::cdc_acm::{self, CdcAcmClass},
};
use gpio::{Level, Output};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

mod resources;
mod system;

use crate::system::{indicator::LEDIndicator, state_machine::StateMachine};
use crate::{
    resources::Irqs,
    system::{System, interface, state_machine},
};
use crate::{
    resources::{AssignedResources, IndicatorResources, InterfaceResources, LoraResources},
    system::indicator,
};

// Program metadata for `picotool info`.
// This isn't needed, but it's recomended to have these minimal entries.
#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"Blinky Example"),
    embassy_rp::binary_info::rp_program_description!(
        c"This example tests the RP Pico on board LED, connected to gpio 25"
    ),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

fn usb_builder<'d, D: embassy_usb_driver::Driver<'d>>(driver: D) -> embassy_usb::Builder<'d, D> {
    let mut usb_config = embassy_usb::Config::new(0xc0de, 0xcafe);
    usb_config.manufacturer = Some("Quanta");
    usb_config.product = Some("Warp");
    usb_config.serial_number = None;
    usb_config.max_power = 100;
    usb_config.max_packet_size_0 = 64;

    static CONFIG_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
    static BOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
    static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();

    embassy_usb::Builder::new(
        driver,
        usb_config.clone(),
        CONFIG_DESCRIPTOR.init([0; 256]),
        BOS_DESCRIPTOR.init([0; 256]),
        &mut [], // no msos descriptors
        CONTROL_BUF.init([0; 64]),
    )
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let r = split_resources!(p);

    state_machine::start(&spawner);

    static INDICATOR_NOTIFIER: indicator::IndicatorNotifier = indicator::notifier();
    static INDICATOR: StaticCell<LEDIndicator<Flex>> = StaticCell::new();
    let indicator = INDICATOR.init(LEDIndicator::new(
        Flex::new(r.indicators.led_pin),
        &INDICATOR_NOTIFIER,
    ));
    spawner
        .spawn(indicator_task(indicator))
        .expect("Failed to spawn indicator module");

    // let mut interface = interface::USBInterface::new("Quanta", "Warp", opts);
    // interface
    //     .start(&spawner, r.interface)https://store.repebble.com/
    //     .expect("Spawner failed.");

    let driver = embassy_rp::usb::Driver::new(r.interface.usb, Irqs);
    let mut builder = usb_builder(driver);

    // Create classes on the builder.
    let acm_class = {
        static STATE: StaticCell<cdc_acm::State> = StaticCell::new();
        let state = STATE.init(cdc_acm::State::new());
        cdc_acm::CdcAcmClass::new(&mut builder, state, 64)
    };

    let (sender, receiver) = acm_class.split();
    static SEND_CHANNEL: Channel<ThreadModeRawMutex, system::Event, 10> = Channel::new();

    let interface = interface::Interface::new(sender, SEND_CHANNEL.receiver());

    // acm_res = spawner.spawn(acm::task(acm_class));

    let usb = builder.build();
    spawner
        .spawn(usb_task(usb))
        .expect("Failed to start usb driver");

    let mut sys = system::System::new(&INDICATOR_NOTIFIER);
    sys.run().await;
}

#[embassy_executor::task]
async fn indicator_task(led: &'static mut LEDIndicator<Flex<'static>>) {
    led.run().await;
}

#[embassy_executor::task]
async fn usb_task(mut usb: UsbDevice<'static, Driver<'static, USB>>) -> ! {
    usb.run().await
}
