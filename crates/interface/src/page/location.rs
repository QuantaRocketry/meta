use buoyant::focus::BoundaryBehavior;
use buoyant::view::prelude::*;
use buoyant::view::scroll_view::ScrollDirection;
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

    let poi = state
        .visible_pois
        .iter()
        .find(|candidate| candidate.id == state.poi_id)
        .cloned()
        .unwrap_or_default();
    let id = poi.id.clone();
    let latitude = poi.coordinate.latitude;
    let longitude = poi.coordinate.longitude;
    let visible_poi_ids: heapless::vec::Vec<heapless::string::String<16>, 6> = state
        .visible_pois
        .iter()
        .map(|poi| poi.id.clone())
        .collect();

    VStack::new((
        header_bar(state),
        Button::new(
            |state: &mut State| state.option_modal = Some(()),
            move |_button_state| {
                HStack::new((
                    Image::new(bearing),
                    VStack::new((
                        grid_row("ID:", Text::new(id.clone(), &font::BODY_FONT)),
                        grid_row(
                            "LAT:",
                            Text::new_fmt::<16>(format_args!("{latitude:.5}"), &font::BODY_FONT),
                        ),
                        grid_row(
                            "LNG:",
                            Text::new_fmt::<16>(format_args!("{longitude:.5}"), &font::BODY_FONT),
                        ),
                    ))
                    .with_spacing(spacing::ELEMENT)
                    .padding(Edges::All, 1),
                ))
                .with_alignment(VerticalAlignment::Top)
                .with_spacing(spacing::COMPONENT)
                .padding(Edges::All, 1)
            },
        )
        .bound_focus(BoundaryBehavior::Wrap),
    ))
    .popover(state.poi_selector.as_ref(), move |_poi_selector| {
        Text::new("POI", &font::FONT)
    })
    .popover(state.option_modal.as_ref(), move |()| {
        poi_modal(visible_poi_ids)
    })
}

/// One row of a two-column label/value grid. The label column is pinned to
/// the width of the widest label so every row's values line up, and each
/// label shares a row with its value so the two can never drift vertically.
fn grid_row<V: View<color::Space, State>>(
    label: &'static str,
    value: V,
) -> impl View<color::Space, State> + use<V> {
    const LABEL_CHARS: u32 = 4;
    let advance = font::BODY_FONT.character_size.width + font::BODY_FONT.character_spacing;

    HStack::new((
        Text::new(label, &font::BODY_FONT)
            .frame()
            .with_width(LABEL_CHARS * advance)
            .with_horizontal_alignment(HorizontalAlignment::Trailing),
        Spacer::default(),
        value,
    ))
}

fn poi_modal(
    visible_poi_ids: heapless::vec::Vec<heapless::string::String<16>, 6>,
) -> impl View<color::Space, State> + use<> {
    const TITLE: &str = "Choose a POI";
    // Bar dimensions kept in sync with the `.with_bar_*` calls below, and the
    // button padding kept in sync with `components::focusable`'s padding, so
    // the scroll view's fixed width leaves exactly enough room for both
    // without clipping the button labels.
    const BAR_WIDTH: u32 = 1;
    const BAR_PADDING: u32 = 1;
    const BUTTON_PADDING: u32 = 3;

    fn poi_button(id: heapless::string::String<16>) -> impl View<color::Space, State> + use<> {
        let selected = id.clone();
        Button::new(
            move |state: &mut State| {
                state.poi_id = selected.clone();
                state.option_modal = None;
            },
            move |button_state| {
                components::focusable(
                    button_state.is_focused(),
                    Text::new(id.clone(), &font::BODY_FONT),
                )
            },
        )
    }

    // The scroll view always claims the full width it's offered, so its
    // width has to be pinned explicitly to the widest label instead of
    // being left to shrink-wrap like a plain VStack would.
    let advance = font::BODY_FONT.character_size.width + font::BODY_FONT.character_spacing;
    let max_chars = visible_poi_ids
        .iter()
        .map(|id| id.len())
        .max()
        .unwrap_or(0)
        .max(TITLE.len()) as u32;
    let content_width = max_chars * advance + BUTTON_PADDING * 2 + BAR_PADDING * 2 + BAR_WIDTH;

    VStack::new((
        Text::new(TITLE, &font::BODY_FONT).padding(Edges::All, spacing::ELEMENT),
        ScrollView::new(
            VStack::new((
                visible_poi_ids.get(0).cloned().map(poi_button),
                visible_poi_ids.get(1).cloned().map(poi_button),
                visible_poi_ids.get(2).cloned().map(poi_button),
                visible_poi_ids.get(3).cloned().map(poi_button),
                visible_poi_ids.get(4).cloned().map(poi_button),
                visible_poi_ids.get(5).cloned().map(poi_button),
            ))
            .with_spacing(spacing::ELEMENT)
            .bound_focus(BoundaryBehavior::Stop),
        )
        .with_direction(ScrollDirection::Vertical)
        .with_bar_width(BAR_WIDTH)
        .with_bar_padding(BAR_PADDING)
        .with_minimum_bar_length(6)
        .frame()
        .with_width(content_width),
    ))
    .with_spacing(spacing::SECTION)
    .padding(Edges::All, spacing::SECTION_MARGIN)
    .background_color(color::BACKGROUND_SECONDARY, RoundedRectangle::new(10))
    .overlay(
        Alignment::Center,
        RoundedRectangle::new(3)
            .stroked_offset(1, StrokeOffset::Outer)
            .foreground_color(color::FOREGROUND_SECONDARY),
    )
    .bound_focus(BoundaryBehavior::Wrap)
}
