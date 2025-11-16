use chrono::Local;
use embedded_graphics::{pixelcolor::Rgb565, prelude::*};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use std::{thread, time::Duration};

fn main() -> Result<(), core::convert::Infallible> {
    let display = SimulatorDisplay::<Rgb565>::new(Size::new(172, 320));

    let output_settings = OutputSettingsBuilder::new().scale(3).build();
    let mut window = Window::new("Entangler", &output_settings);

    let theme = common::DisplayTheme::default();
    let mut entangler_display = common::Display::new(display, theme, 10);

    'running: loop {
        let time = Local::now();
        let _ = entangler_display.draw(time);

        window.update(entangler_display.get_target());

        if window.events().any(|e| e == SimulatorEvent::Quit) {
            break 'running Ok(());
        }
        thread::sleep(Duration::from_millis(50));
    }
}
