use buoyant::view::prelude::*;
use embedded_graphics::{
    mono_font::{MonoFont, ascii::FONT_10X20},
    pixelcolor::Rgb565,
    prelude::*,
};

#[cfg(feature = "rgb565")]
pub type COLOR_SPACE = Rgb565;

#[cfg(feature = "rgb888")]
pub type COLOR_SPACE = Rgb565;

pub struct State {
    page: Page,
}

impl State {
    pub fn new() -> Self {
        State {
            page: Page::Radar,
        }
    }
}

enum Page {
    Radar,
}

#[derive(Clone)]
pub struct Theme<C> {
    pub primary: C,
    pub secondary: C,
    pub background: C,
}

impl<C: RgbColor> Default for Theme<C> {
    fn default() -> Self {
        Self {
            primary: RgbColor::BLACK,
            secondary: RgbColor::BLACK,
            background: RgbColor::WHITE,
        }
    }
}

pub fn view(state: State, theme: &Theme<COLOR_SPACE>) -> impl View<COLOR_SPACE> {
    match state.page {
        Page::Radar => HStack::new((
            Text::new("Hello", &FONT_10X20).foreground_color(Rgb565::GREEN),
            Spacer::default(),
            Text::new("World", &FONT_10X20).foreground_color(Rgb565::YELLOW),
        ))
        .padding(Edges::All, 20),
    }
}
