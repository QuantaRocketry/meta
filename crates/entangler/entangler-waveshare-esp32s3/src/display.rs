use embassy_sync::{channel::DynamicReceiver, signal::Signal};
use embedded_graphics_core::pixelcolor::Rgb565;
use embedded_hal::{delay::DelayNs, digital::OutputPin, spi::SpiDevice};

extern crate alloc;
use alloc::vec::Vec;

use mipidsi::{
    dcs::{
        BitsPerPixel, EnterNormalMode, ExitSleepMode, InterfaceExt, PixelFormat, SetAddressMode,
        SetDisplayOn, SetInvertMode, SetPixelFormat,
    },
    interface::SpiInterface,
    models::Model,
};

pub mod model;

pub type ColorFormat = Rgb565;
pub const DISPLAY_WIDTH: u32 = 172;
pub const DISPLAY_HEIGHT: u32 = 320;

// type ImplSpiDevice = impl embedded_hal::spi::SpiDevice;
// type ImplOutputPin = impl embedded_hal::digital::OutputPin;

// #[embassy_executor::task]
// async fn render_task(
//     rx_render: DynamicReceiver<'static, Reusable<'static, Vec<ColorFormat>>>,
//     spi: ImplSpiDevice,
//     dc: ImplOutputPin,
// ) {
//     // let sdl_context = sdl2::init()?;
//     // let video_subsystem = sdl_context.video()?;

//     // let window = video_subsystem
//     //     .window("Demo", DISPLAY_WIDTH as _, DISPLAY_HEIGHT as _)
//     //     .position_centered()
//     //     .opengl()
//     //     .build()
//     //     .map_err(|e| e.to_string())?;

//     // let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
//     // let texture_creator = canvas.texture_creator();

//     // let mut texture = texture_creator
//     //     .create_texture_streaming(
//     //         PixelFormatEnum::RGB565,
//     //         DISPLAY_WIDTH as _,
//     //         DISPLAY_HEIGHT as _,
//     //     )
//     //     .map_err(|e| e.to_string())?;

//     // canvas.clear();
//     // canvas.copy(
//     //     &texture,
//     //     None,
//     //     Some(Rect::new(0, 0, DISPLAY_WIDTH as _, DISPLAY_HEIGHT as _)),
//     // )?;
//     // canvas.present();

//     // let mut event_pump = sdl_context.event_pump()?;

//     // loop {
//     // for event in event_pump.poll_iter() {
//     //     match event {
//     //         Event::Quit { .. }
//     //         | Event::KeyDown {
//     //             keycode: Some(Keycode::Escape),
//     //             ..
//     //         } => std::process::exit(0),
//     //         Event::KeyDown {
//     //             keycode: Some(Keycode::LSHIFT),
//     //             ..
//     //         } => controller::send_action(Action::HardwareUserBtnPressed(true)),
//     //         Event::KeyUp {
//     //             keycode: Some(Keycode::LSHIFT),
//     //             ..
//     //         } => controller::send_action(Action::HardwareUserBtnPressed(false)),
//     //         Event::MouseButtonDown {
//     //             timestamp: _timestamp,
//     //             window_id: _window_id,
//     //             which: _which,
//     //             mouse_btn,
//     //             clicks: _clicks,
//     //             x,
//     //             y,
//     //         } => {
//     //             if mouse_btn == MouseButton::Left {
//     //                 let button = PointerEventButton::Left;
//     //                 let position = slint::PhysicalPosition::new(x, y).to_logical(1.0);
//     //                 let event = WindowEvent::PointerPressed { position, button };
//     //                 tx_event.send(event).unwrap();
//     //             }
//     //         }
//     //         Event::MouseButtonUp {
//     //             timestamp: _timestamp,
//     //             window_id: _window_id,
//     //             which: _which,
//     //             mouse_btn,
//     //             clicks: _clicks,
//     //             x,
//     //             y,
//     //         } => {
//     //             if mouse_btn == MouseButton::Left {
//     //                 let button = PointerEventButton::Left;
//     //                 let position = slint::PhysicalPosition::new(x, y).to_logical(1.0);
//     //                 let event = WindowEvent::PointerReleased { position, button };
//     //                 tx_event.send(event).unwrap();
//     //             }
//     //         }
//     //         Event::MouseMotion {
//     //             timestamp: _timestamp,
//     //             window_id: _window_id,
//     //             which: _which,
//     //             mousestate,
//     //             x,
//     //             y,
//     //             xrel: _xrel,
//     //             yrel: _yrel,
//     //         } => {
//     //             if mousestate.is_mouse_button_pressed(MouseButton::Left) {
//     //                 let position = slint::PhysicalPosition::new(x, y).to_logical(1.0);
//     //                 let event = WindowEvent::PointerMoved { position };
//     //                 tx_event.send(event).unwrap();
//     //             }
//     //         }

//     //         _ => {}
//     //     }
//     // }

//     'render_buffers: loop {
//         match rx_render.try_recv() {
//             Ok(buf) => {
//                 // texture.with_lock(None, |buffer: &mut [u8], _pitch: usize| {
//                 //     let buf_ptr = buf.as_ptr() as *const u8;
//                 //     let buf_slice = unsafe { slice::from_raw_parts(buf_ptr, buf.len() * 2) };
//                 //     buffer.copy_from_slice(buf_slice);
//                 //     drop(buf); // returns buffer to pool
//                 // })?;
//                 // canvas.clear();
//                 // canvas.copy_ex(
//                 //     &texture,
//                 //     None,
//                 //     Some(Rect::new(0, 0, DISPLAY_WIDTH as _, DISPLAY_HEIGHT as _)),
//                 //     0.0,
//                 //     None,
//                 //     false,
//                 //     false,
//                 // )?;
//                 // canvas.present();
//             }
//             _ => {
//                 // ignore
//                 // break 'render_buffers;
//             }
//         }
//     }
//     // }
// }
