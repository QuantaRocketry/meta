use buoyant::{transition::Move, view::prelude::*};
use embedded_graphics::pixelcolor::BinaryColor;

use crate::{State, color, spacing};

/// Wraps `base` with a centered modal card that appears whenever `value` is `Some`.
///
/// `content` and `lens` only ever deal with `T` -- the slice of `State` this particular
/// modal edits -- so the same `modal()` can back any field of `State`: only `T`, `lens`,
/// and `content` change between call sites, while the popover wiring and card styling
/// (background, border, centering, transition) stay shared.
///
/// `lens` must always return a valid `&mut T` even though it's only actually read while
/// `value` is `Some`; when there's no natural "always there" backing value, fall back to
/// a default/last-known value (see `clean_settings` alongside `clean_overlay` in
/// `InterfaceState`).
pub fn modal<T, V>(
    base: impl View<BinaryColor, State>,
    value: Option<&T>,
    lens: impl Fn(&mut State) -> &mut T + Clone,
    content: impl FnOnce(&T) -> V,
) -> impl View<BinaryColor, State>
where
    T: Clone,
    V: View<BinaryColor, T>,
{
    base.popover(value, move |value: &T| {
        Lens::new(content(value), lens)
            .padding(Edges::All, spacing::COMPONENT)
            .background_color(color::BACKGROUND_SECONDARY, RoundedRectangle::new(10))
            .overlay(
                Alignment::Center,
                RoundedRectangle::new(10)
                    .stroked_offset(2, StrokeOffset::Outer)
                    .foreground_color(color::FOREGROUND_SECONDARY),
            )
            .transition(Move::top())
    })
}
