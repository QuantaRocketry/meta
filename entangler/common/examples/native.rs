use buoyant::view::AsDrawable;
use common::app::{self, COLOR_SPACE};
use embedded_graphics::{pixelcolor::Rgb565, prelude::*};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use std::{thread, time::Duration};

fn main() -> Result<(), core::convert::Infallible> {
    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(172, 320));

    let output_settings = OutputSettingsBuilder::new().scale(2).build();
    let mut window = Window::new("Entangler", &output_settings);

    let theme: common::app::Theme<COLOR_SPACE> = common::app::Theme {
        primary: COLOR_SPACE::BLACK,
        secondary: COLOR_SPACE::CSS_GRAY,
        background: COLOR_SPACE::WHITE,
    };
    // let mut entangler_display = common::Display::new(display, theme.clone(), 10);

    // let signals = [
    //     common::Signal {
    //         uid: "VK2-XXX",
    //         snr: 10,
    //         rssi: -20,
    //         angle: 0.0,
    //         distance: 325.0,
    //     },
    //     common::Signal {
    //         uid: "OTHER",
    //         snr: 10,
    //         rssi: -20,
    //         angle: -130.0,
    //         distance: 1000.0,
    //     },
    // ];

    // entangler_display.select_signal(&signals[0]);

    'running: loop {
        // let _ = entangler_display.draw(&signals);
        let v = common::app::view(app::State::new(), &theme);
        v.as_drawable(display.size(), Rgb565::WHITE).draw(&mut display).unwrap();

        window.update(&display);

        if window.events().any(|e| e == SimulatorEvent::Quit) {
            break 'running Ok(());
        }
        thread::sleep(Duration::from_millis(50));
    }
}
