use buoyant::focus::BoundaryBehavior;
use buoyant::view::prelude::*;
use embedded_graphics::pixelcolor::BinaryColor;

extern crate alloc;

use crate::components::{self, header_bar};
use crate::{State, color, font, spacing};

pub fn view(state: &State) -> impl View<color::Space, State> + use<> {
    static mut BEARING: components::Bearing<BinaryColor> = components::Bearing {
        color: BinaryColor::On,
        angle: 0.0,
    };

    // SAFETY: `view` is not called concurrently or re-entrantly (e.g. from
    // an interrupt handler while already running), so this momentary
    // exclusive access to the mutable static is sound, and the shared
    // reference handed out below is the only live reference to it.
    let bearing: &'static components::Bearing<BinaryColor> = unsafe {
        let ptr = &raw mut BEARING;
        (*ptr).angle = state.heading;
        &*ptr
    };

    let id = state.poi.id.clone();
    let latitude = &state.poi.latitude;
    let longitude = &state.poi.longitude;
    VStack::new((
        header_bar(state),
        HStack::new((
            Image::new(bearing),
            HStack::new((
                VStack::new((
                    Text::new("ID:", &font::BODY_FONT),
                    Text::new("LAT:", &font::BODY_FONT),
                    Text::new("LNG:", &font::BODY_FONT),
                ))
                .with_alignment(HorizontalAlignment::Trailing)
                .with_spacing(spacing::ELEMENT),
                Spacer::default(),
                VStack::new((
                    Text::new(id, &font::BODY_FONT),
                    Text::new_fmt::<16>(format_args!("{latitude:.5}"), &font::BODY_FONT),
                    Text::new_fmt::<16>(format_args!("{longitude:.5}"), &font::BODY_FONT),
                ))
                .with_alignment(HorizontalAlignment::Trailing)
                .with_spacing(spacing::ELEMENT),
            ))
            .padding(Edges::All, 1),
        ))
        .with_alignment(VerticalAlignment::Top)
        .with_spacing(spacing::COMPONENT)
        .padding(Edges::All, 1),
    ))
    .bound_focus(BoundaryBehavior::Wrap)
    .popover(state.poi_selector.as_ref(), move |_poi_selector| {
        Text::new("POI", &font::FONT)
    })
}
