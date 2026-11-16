use buoyant::view::prelude::*;
use buoyant::{layout::Alignment, view::View};
use embedded_graphics::pixelcolor::BinaryColor;

mod battery;
mod bearing;
mod modal;

pub use bearing::Bearing;

use crate::{State, color, font};

pub fn focusable(
    is_focused: bool,
    inner: impl View<color::Space, State>,
) -> impl View<color::Space, State> {
    let inner = inner.padding(Edges::All, 3);
    buoyant::if_view!((is_focused) {
        inner.background(
            Alignment::Center,
            RoundedRectangle::new(3)
                .stroked(1)
                .foreground_color(BinaryColor::On)
        )
    } else {
        inner
    })
}

pub fn header_bar(state: &State) -> impl View<color::Space, State> + use<> {
    HStack::new((
        Text::new(state.page.as_str(), &font::HEADER_FONT).padding(Edges::Leading, 2),
        Spacer::default(),
        battery::widget(state).padding(Edges::Horizontal, 2),
    ))
    .padding(Edges::Vertical, 1)
    .foreground_color(color::Space::Off)
    .background_color(color::Space::On, RoundedRectangle::new(4))
}
