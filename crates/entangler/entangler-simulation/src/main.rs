extern crate alloc;
use alloc::{rc::Rc, vec::Vec};

use embassy_executor::{Executor, Spawner};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    channel::{Channel, DynamicReceiver, DynamicSender},
};
use embassy_time::{Duration, Timer};
use embedded_graphics::{
    pixelcolor::{raw::RawU16, Rgb565},
    prelude::*,
};
use embedded_graphics_simulator::{
    sdl2::{Keycode, MouseButton},
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use entangler_common::{
    controller::{self, Action, Controller},
    ui::MainWindow,
};
use entangler_simulation::{
    simulator::hardware::HardwareSim,
    slint_backend::{StmBackend, TargetPixelType, DISPLAY_HEIGHT, DISPLAY_WIDTH},
};
use log::*;
use object_pool::{Pool, Reusable};
use slint::{
    platform::{
        software_renderer::{MinimalSoftwareWindow, RepaintBufferType},
        PointerEventButton, WindowAdapter, WindowEvent,
    },
    ComponentHandle, PhysicalPosition,
};
use static_cell::StaticCell;

static EXECUTOR: StaticCell<Executor> = StaticCell::new();
static POOL: StaticCell<Pool<Vec<TargetPixelType>>> = StaticCell::new();

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .format_timestamp_nanos()
        .init();

    static C_RENDER: Channel<CriticalSectionRawMutex, Reusable<'static, Vec<TargetPixelType>>, 4> =
        Channel::new();
    static C_EVENT: Channel<CriticalSectionRawMutex, WindowEvent, 16> = Channel::new();

    let pool = POOL.init(Pool::new(4, || {
        vec![TargetPixelType::default(); DISPLAY_WIDTH * DISPLAY_HEIGHT]
    }));

    let executor = EXECUTOR.init(Executor::new());
    executor.run(|spawner| {
        spawner
            .spawn(sim_render_loop(
                C_RENDER.dyn_receiver(),
                C_EVENT.dyn_sender(),
            ))
            .unwrap();
        spawner
            .spawn(main_task(
                spawner,
                C_RENDER.dyn_sender(),
                C_EVENT.dyn_receiver(),
                pool,
            ))
            .unwrap();
    });
}

#[embassy_executor::task]
async fn main_task(
    spawner: Spawner,
    tx_render: DynamicSender<'static, Reusable<'static, Vec<TargetPixelType>>>,
    rx_event: DynamicReceiver<'static, WindowEvent>,
    pool: &'static Pool<Vec<TargetPixelType>>,
) {
    
    let backend = Box::new(StmBackend::new(window.clone()));
    slint::platform::set_platform(backend).expect("backend already initialized");
    info!("slint gui setup complete");

    spawner
        .spawn(embassy_render_loop(window, tx_render, rx_event, pool))
        .unwrap();

    // give the render loop time to come up (otherwise it will draw a blank screen)
    Timer::after(Duration::from_millis(200)).await;
    let main_window = MainWindow::new().unwrap();
    main_window.show().expect("unable to show main window");

    info!("press LEFT SHIFT to simulate a hardware button press");

    let hardware = HardwareSim {};

    // run the gui controller loop
    let mut controller = Controller::new(&main_window, hardware);
    controller.run().await;
}

#[embassy_executor::task]
async fn embassy_render_loop(
    window: Rc<MinimalSoftwareWindow>,
    tx_render: DynamicSender<'static, Reusable<'static, Vec<TargetPixelType>>>,
    rx_event: DynamicReceiver<'static, WindowEvent>,
    pool: &'static Pool<Vec<TargetPixelType>>,
) {
    info!("embassy_render_loop");

    loop {
        slint::platform::update_timers_and_animations();
        'event: loop {
            match rx_event.try_receive() {
                Ok(e) => {
                    window.dispatch_event(e);
                }
                Err(_) => break 'event,
            }
        }

        // redraw the entire window (otherwise we get partial redraws which are more complicated to deal with)
        window.request_redraw();

        let _is_dirty = window.draw_if_needed(|renderer| match pool.try_pull() {
            Some(mut buffer) => {
                renderer.render(&mut buffer, DISPLAY_WIDTH as _);
                if let Err(_) = tx_render.try_send(buffer) {
                    panic!("Failed to tx render buffer");
                }
            }
            None => {
                // this happens when the MainWindow hasn't yet been created or if it has been closed by the user
            }
        });

        // for approx 60fps
        Timer::after(Duration::from_millis(16)).await;
    }
}

#[embassy_executor::task]
async fn sim_render_loop(
    rx_render: DynamicReceiver<'static, Reusable<'static, Vec<TargetPixelType>>>,
    tx_event: DynamicSender<'static, WindowEvent>,
) {
    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(172, 320));

    let output_settings = OutputSettingsBuilder::new().scale(2).build();
    let mut window = Window::new("Hello World", &output_settings);
    window.update(&display);
    loop {
        for event in window.events() {
            match event {
                // Handle Window Close/Escape Key
                SimulatorEvent::Quit
                | SimulatorEvent::KeyDown {
                    keycode: Keycode::Escape,
                    ..
                } => std::process::exit(0),

                // Map Key Events for hardware button (LSHIFT in original code)
                SimulatorEvent::KeyDown {
                    keycode: Keycode::LShift,
                    ..
                } => controller::send_action(Action::HardwareUserBtnPressed(true)),
                SimulatorEvent::KeyUp {
                    keycode: Keycode::LShift,
                    ..
                } => controller::send_action(Action::HardwareUserBtnPressed(false)),

                // Map Mouse/Pointer Events
                SimulatorEvent::MouseButtonDown { mouse_btn, point } => {
                    if mouse_btn == MouseButton::Left {
                        let position = PhysicalPosition::new(point.x, point.y).to_logical(1.0);
                        let event = WindowEvent::PointerPressed {
                            position,
                            button: PointerEventButton::Left,
                        };
                        if tx_event.try_send(event).is_err() {
                            eprintln!("Error sending PointerPressed event");
                        }
                    }
                }
                SimulatorEvent::MouseButtonUp { mouse_btn, point } => {
                    if mouse_btn == MouseButton::Left {
                        let position = PhysicalPosition::new(point.x, point.y).to_logical(1.0);
                        let event = WindowEvent::PointerReleased {
                            position,
                            button: PointerEventButton::Left,
                        };
                        if tx_event.try_send(event).is_err() {
                            eprintln!("Error sending PointerReleased event");
                        }
                    }
                }
                SimulatorEvent::MouseMove { point } => {
                    let position = PhysicalPosition::new(point.x, point.y).to_logical(1.0);
                    let event = WindowEvent::PointerMoved { position };
                    if tx_event.try_send(event).is_err() {
                        eprintln!("Error sending PointerMoved event");
                    }
                }
                // Ignore other events (Resize, other Keycodes, etc.)
                _ => {}
            }
        }

        'render_buffers: loop {
            match rx_render.try_receive() {
                Ok(buf) => {
                    let pixel_iter = buf
                        .as_slice()
                        .iter()
                        // Use .copied() since Rgb565Pixel is usually Copy
                        .copied()
                        .enumerate()
                        .map(|(index, color)| {
                            let x = (index as u32) % DISPLAY_WIDTH as u32;
                            let y = (index as u32) / DISPLAY_WIDTH as u32;
                            let color = Rgb565::from(RawU16::new(color.0));

                            Pixel(Point::new(x as i32, y as i32), color)
                        });

                    // Draw the iterator onto the SimulatorDisplay, which implements DrawTarget.
                    if let Err(e) = display.draw_iter(pixel_iter) {
                        eprintln!("Error drawing to simulator display: {:?}", e);
                    }

                    window.update(&display);
                }
                _ => {
                    // ignore
                    break 'render_buffers;
                }
            }
        }
        Timer::after(Duration::from_millis(1)).await;
    }
}
