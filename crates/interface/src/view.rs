use buoyant::view::prelude::*;
use buoyant::view::scroll_view::ScrollDirection;
use embedded_graphics::pixelcolor::BinaryColor;

use super::{State, color, font, spacing};

pub mod clean;

pub fn brew_tab(_state: &State) -> impl View<BinaryColor, State> + use<> {
    ScrollView::new(
        VStack::new((
            Text::new("Good morning", &font::FONT),
            Text::new(
                "You can't brew coffee in a simulator, but you can pretend.",
                &font::FONT,
            )
            .multiline_text_alignment(HorizontalTextAlignment::Center),
        ))
        .with_spacing(spacing::COMPONENT)
        .with_alignment(HorizontalAlignment::Center)
        .flex_infinite_width(HorizontalAlignment::Center)
        .padding(Edges::All, spacing::SECTION_MARGIN)
        .foreground_color(color::Space::On),
    )
    .with_direction(ScrollDirection::Both)
}

pub fn settings_tab(_state: &State) -> impl View<BinaryColor, State> + use<> {
    ScrollView::new(
        VStack::new((
            Text::new("Good morning", &font::FONT),
            Text::new(
                "You can't brew coffee in a simulator, but you can pretend.",
                &font::FONT,
            )
            .multiline_text_alignment(HorizontalTextAlignment::Center),
        ))
        .with_spacing(spacing::COMPONENT)
        .with_alignment(HorizontalAlignment::Center)
        .flex_infinite_width(HorizontalAlignment::Center)
        .padding(Edges::All, spacing::SECTION_MARGIN)
        .foreground_color(BinaryColor::On),
    )
    .with_direction(ScrollDirection::Vertical)
}
