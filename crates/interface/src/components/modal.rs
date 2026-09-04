use buoyant::view::{Text, View};
use embedded_graphics::pixelcolor::BinaryColor;

use crate::{State, font::FONT};

pub fn modal(_inner: impl View<BinaryColor, State>) -> impl View<BinaryColor, State> {
    Text::new("aksjhkd", &FONT)
}
