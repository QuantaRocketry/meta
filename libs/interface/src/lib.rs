#![no_std]

use buoyant::{
    event::Event,
    focus::{self, FocusAction},
    view::{map_event::Mapping, prelude::*},
};

mod components;
pub mod event;
mod icons;
mod page;
mod state;

pub use qcp::TrackedPOI;
pub use state::InterfaceState as State;

use page::Page;

#[allow(unused)]
pub mod spacing {
    /// Spacing between sections / groups
    pub const SECTION: u32 = 2;
    /// Outer padding to the edge of the screen
    pub const SECTION_MARGIN: u32 = 2;
    /// Spacing between distinct visual components in a section / group
    pub const COMPONENT: u32 = 2;
    /// Spacing between elements within a component
    pub const ELEMENT: u32 = 1;
}

#[allow(unused)]
pub mod color {
    use embedded_graphics::prelude::*;

    pub type Space = embedded_graphics::pixelcolor::BinaryColor;
    pub const ACCENT: Space = Space::Off;
    pub const BACKGROUND: Space = Space::Off;
    pub const BACKGROUND_SECONDARY: Space = Space::Off;
    pub const FOREGROUND_SECONDARY: Space = Space::On;
}

#[allow(unused)]
pub mod font {
    use super::color;

    use embedded_graphics::{
        mono_font::{MonoFont, ascii::*},
        prelude::RgbColor as _,
    };
    // use embedded_ttf::{FontTextStyle, FontTextStyleBuilder};
    use u8g2_fonts::{FontRenderer, fonts};

    pub static FONT: FontRenderer = FontRenderer::new::<fonts::u8g2_font_5x7_tr>();
    pub static BODY_FONT: MonoFont<'_> = embedded_graphics::mono_font::ascii::FONT_5X7;
    pub static HEADER_FONT: FontRenderer = FontRenderer::new::<fonts::u8g2_font_5x7_tr>();
    pub static CAPTION_SIZE: u32 = 14;
    pub static BODY_SIZE: u32 = 20;
    pub static HEADING_SIZE: u32 = 32;
}

pub fn view(state: &State) -> impl View<color::Space, State> + use<> {
    let paginate = |s: &mut State, event| {
        match (event, s.page) {
            (buoyant::view::paginate::PageEvent::Next, Page::Location) => s.page = Page::Settings,
            (buoyant::view::paginate::PageEvent::Next, Page::Settings) => s.page = Page::Location,
            (buoyant::view::paginate::PageEvent::Previous, Page::Location) => {
                s.page = Page::Settings
            }
            (buoyant::view::paginate::PageEvent::Previous, Page::Settings) => {
                s.page = Page::Location
            }
        }
        // Popovers belong to the page they were opened on.
        s.clean_overlay = None;
        s.poi_selector = None;
        s.option_modal = None;
    };

    buoyant::view::Paginate::new(focus::GROUP_1, paginate, {
        buoyant::match_view!(state.page, {
            Page::Settings => page::settings::view(state),
            Page::Location => page::location::view(state),
        })
    })
    .focus_touches()
    .map_event(|event: &Event, _state| match event {
        Event::KeyDown(key) => match key {
            buoyant::event::Key::LeftArrow => {
                Mapping::Fallback(FocusAction::Previous.into_event(focus::GROUP_1))
            }
            buoyant::event::Key::RightArrow => {
                Mapping::Fallback(FocusAction::Next.into_event(focus::GROUP_1))
            }
            buoyant::event::Key::UpArrow => {
                Mapping::Fallback(FocusAction::Previous.into_event(focus::GROUP_0))
            }
            buoyant::event::Key::DownArrow => {
                Mapping::Fallback(FocusAction::Next.into_event(focus::GROUP_0))
            }
            buoyant::event::Key::Character(' ' | '\n') => {
                Mapping::Fallback(FocusAction::Select.into_event(focus::GROUP_0))
            }
            buoyant::event::Key::Escape => {
                Mapping::Fallback(FocusAction::Blur.into_event(focus::GROUP_0))
            }
            _ => Mapping::Passthrough,
        },
        _ => Mapping::Passthrough,
    })
}
