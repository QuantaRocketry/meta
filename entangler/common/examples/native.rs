use chrono::Local;
use embedded_graphics::{pixelcolor::Rgb565, prelude::*};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use std::{thread, time::Duration};

fn main() -> Result<(), core::convert::Infallible> {
    let display = SimulatorDisplay::<Rgb565>::new(Size::new(172, 320));

    let output_settings = OutputSettingsBuilder::new().scale(2).build();
    let mut window = Window::new("Entangler", &output_settings);

    let theme = common::DisplayTheme {
        primary: Rgb565::BLACK,
        secondary: Rgb565::CSS_GRAY,
        background: Rgb565::WHITE,
    };
    let mut entangler_display = common::Display::new(display, theme, 10);

    let signals = [common::Signal {
        uid: "VK2-XXX",
        snr: 10,
        rssi: -20,
        angle: 0.0,
        distance: 325.0,
    },
    common::Signal {
        uid: "OTHER",
        snr: 10,
        rssi: -20,
        angle: -130.0,
        distance: 1000.0,
    }];

    entangler_display.select_signal(&signals[0]);

    'running: loop {
        let time = Local::now();
        let _ = entangler_display.draw(&signals);

        window.update(entangler_display.get_target());

        if window.events().any(|e| e == SimulatorEvent::Quit) {
            break 'running Ok(());
        }
        thread::sleep(Duration::from_millis(50));
    }
}
