#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio;
use embassy_time::Timer;
use embassy_usb::Config;
use gpio::{Level, Output};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

mod resources;
mod system;

use crate::system::{System, interface, state_machine};
use crate::system::{indicator::LEDIndicator, state_machine::StateMachine};
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

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let r = split_resources!(p);

    state_machine::start(&spawner);
    LEDIndicator::start(&spawner, r.indicators).await;

    let opts = interface::USBOpts {acm: true, ncm: false};
    let mut interface = interface::USBInterface::new("Quanta", "Warp", opts);
    interface.start(&spawner, r.interface).expect("Spawner failed.");

    system::start(&spawner).await.expect("Spawner failed.");
}
