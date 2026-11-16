use buoyant::view::prelude::*;
use embedded_graphics::pixelcolor::BinaryColor;

use crate::{State, color, font, icons, spacing};

pub fn widget(state: &State) -> impl View<color::Space, State> + use<> {
    static mut BATTERY: icons::BatteryIcon<BinaryColor> = icons::BatteryIcon {
        color: BinaryColor::Off,
        charge: 0.5,
    };

    // SAFETY: `view` is not called concurrently or re-entrantly (e.g. from
    // an interrupt handler while already running), so this momentary
    // exclusive access to the mutable static is sound, and the shared
    // reference handed out below is the only live reference to it.
    let battery: &'static icons::BatteryIcon<BinaryColor> = unsafe {
        let ptr = &raw mut BATTERY;
        (*ptr).charge = state.battery_percentage;
        &*ptr
    };

    let percent = libm::roundf(state.battery_percentage * 100.0) as u8;

    HStack::new((
        Text::new_fmt::<8>(format_args!("{percent}%"), &font::HEADER_FONT),
        Image::new(battery),
    ))
    .with_spacing(spacing::ELEMENT)
}
