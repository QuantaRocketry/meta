use buoyant::focus::BoundaryBehavior;
use buoyant::view::prelude::*;

use crate::components::header_bar;
use crate::{State, color, font};

pub fn view(state: &State) -> impl View<color::Space, State> + use<> {
    VStack::new((header_bar(state), Text::new("set 1", &font::BODY_FONT)))
        .bound_focus(BoundaryBehavior::Stop)
}
