use buoyant::app::Harness as _;
use buoyant::event::{self, Event, Key};
use buoyant::render_target::{EmbeddedGraphicsRenderTarget, RenderTarget as _};
use embedded_graphics::{pixelcolor::BinaryColor, prelude::*};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window, sdl2::Keycode,
};

use std::thread;
use std::time::{Duration, Instant};

// Import your shared UI library
use interface::event::{InterfaceEvent, NavigateEvent};
use interface::{State, view};

fn main() -> Result<(), core::convert::Infallible> {
    // Using 128x64 to match your Sh1106_128_64 hardware
    let mut display: SimulatorDisplay<BinaryColor> = SimulatorDisplay::new(Size::new(128, 64));

    let output_settings = OutputSettingsBuilder::new()
        .scale(4) // Scale up 4x for easier viewing on standard monitors
        .theme(embedded_graphics_simulator::BinaryColorTheme::OledBlue)
        .build();
    let mut window = Window::new("Wio L1 Simulator", &output_settings);

    let size = display.size().into();
    let mut app = buoyant::app::App::new(State::default(), size, view);
    let mut s = app.state_mut();
    s.poi.id.insert_str(0, "VK2GTX").unwrap();
    s.poi.latitude = 33.123123;
    s.poi.longitude = -151.456456;

    let mut target = EmbeddedGraphicsRenderTarget::new_hinted(&mut display, BinaryColor::On);

    // Force an update before checking events
    window.update(target.display_mut());

    let start_time = Instant::now();

    'running: loop {
        // Sync app time with real wall clock time
        app.set_time(start_time.elapsed());

        // Set state based on 'system events' (mock data)
        {
            let mut s = app.state_mut();
            s.heading = ((start_time.elapsed().as_secs_f32() / 2.0).sin() + 1.0) * 180.0;
            s.battery_percentage = (start_time.elapsed().as_secs_f32().sin() + 1.0) / 2.0;
        }

        for event in window.events() {
            let app_event = match event {
                SimulatorEvent::Quit => break 'running,

                // Map keyboard presses to Buoyant navigation/events
                SimulatorEvent::KeyDown { keycode, .. } => match keycode {
                    Keycode::Up => Some(Event::KeyDown(Key::UpArrow)),
                    Keycode::Down => Some(Event::KeyDown(Key::DownArrow)),
                    Keycode::Left => Some(Event::KeyDown(Key::LeftArrow)),
                    Keycode::Right => Some(Event::KeyDown(Key::RightArrow)),
                    Keycode::Return => Some(Event::KeyDown(Key::Character('\n'))),
                    Keycode::Escape => Some(Event::KeyDown(Key::Escape)),
                    _ => None,
                },
                SimulatorEvent::KeyUp { keycode, .. } => match keycode {
                    Keycode::Up => Some(Event::KeyUp(Key::UpArrow)),
                    Keycode::Down => Some(Event::KeyUp(Key::DownArrow)),
                    Keycode::Left => Some(Event::KeyUp(Key::LeftArrow)),
                    Keycode::Right => Some(Event::KeyUp(Key::RightArrow)),
                    Keycode::Return => Some(Event::KeyUp(Key::Character('\n'))),
                    Keycode::Escape => Some(Event::KeyUp(Key::Escape)),
                    _ => None,
                },
                _ => None,
            };

            // If we successfully mapped an event, send it to the app
            if let Some(e) = app_event {
                app.send(e);
            }
        }

        if app.should_redraw() || target.clear_animation_status() {
            // Render animated transition between source and target trees
            app.render_animated(&mut target, &BinaryColor::On);

            // Draw focus overlay
            // app.draw_focus_overlay(&mut target, BinaryColor::On, 1);

            // Send to the simulator window (Replaces target.display_mut().flush().await)
            window.update(target.display_mut());

            // Clear for the next frame
            target.clear(BinaryColor::Off);
        } else {
            // Limit polling for updates to ~30 fps when idle (Replaces Timer::after_millis)
            thread::sleep(Duration::from_millis(33));
        }
    }

    Ok(())
}
