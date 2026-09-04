use core::{cell::RefCell, error};

use embassy_sync::{
    blocking_mutex::{Mutex, NoopMutex, raw::NoopRawMutex},
    channel::{DynamicReceiver, DynamicSender},
};
use embassy_time::{Duration, Timer};
use embedded_graphics::{
    Pixel,
    draw_target::{DrawTarget, DrawTargetExt},
    pixelcolor::Rgb565,
    prelude::*,
};

use alloc::{boxed::Box, rc::Rc, sync::Arc, vec::*};
use opool::{Pool, PoolAllocator, RcGuard};
use slint::platform::{
    PointerEventButton, WindowAdapter, WindowEvent,
    software_renderer::{MinimalSoftwareWindow, RepaintBufferType, Rgb565Pixel, TargetPixel},
};

use crate::*;

mod event;
mod state;
pub use event::Event;
pub use state::State;

slint::include_modules!();

const DISPLAY_HEIGHT: usize = 320;
const DISPLAY_WIDTH: usize = 172;

struct SlintPoolAllocator;
impl PoolAllocator<Vec<Rgb565Pixel>> for SlintPoolAllocator {
    fn allocate(&self) -> Vec<Rgb565Pixel> {
        Vec::with_capacity(DISPLAY_HEIGHT * DISPLAY_WIDTH)
    }

    #[inline]
    fn reset(&self, obj: &mut Vec<Rgb565Pixel>) {
        obj.clear();
    }
}

struct SlintFramebuffer<'a> {
    pub buffer: &'a mut [Rgb565Pixel],
    pub size: Size,
}

impl<'a> DrawTarget for SlintFramebuffer<'a> {
    type Color = Rgb565;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels {
            // Check bounds to prevent panics
            if point.x >= 0
                && point.x < self.size.width as i32
                && point.y >= 0
                && point.y < self.size.height as i32
            {
                let index = (point.y as usize * self.size.width as usize) + point.x as usize;
                self.buffer[index] = Rgb565Pixel::from_rgb(color.r(), color.g(), color.b());
            }
        }
        Ok(())
    }
}

impl<'a> OriginDimensions for SlintFramebuffer<'a> {
    fn size(&self) -> Size {
        self.size
    }
}

struct Ui {
    slint_window: Rc<MinimalSoftwareWindow>,
    pool: Arc<Pool<SlintPoolAllocator, Vec<Rgb565Pixel>>>,
    state: RefCell<State>,
}

impl Ui {
    fn new(width: u32, height: u32) -> Self {
        let slint_window = MinimalSoftwareWindow::new(RepaintBufferType::SwappedBuffers);
        slint_window.set_size(slint::PhysicalSize::new(width, height));
        let pool = Pool::new_prefilled(2, SlintPoolAllocator).to_rc();
        let state = RefCell::new(State::default());
        Ui {
            slint_window,
            pool,
            state,
        }
    }

    async fn run_slint(
        &self,
        rx_event: DynamicReceiver<'static, ui::Event>,
        tx_render: DynamicSender<'static, opool::RcGuard<SlintPoolAllocator, Vec<Rgb565Pixel>>>,
    ) {
        loop {
            slint::platform::update_timers_and_animations();
            while let Ok(e) = rx_event.try_receive() {
                match e {
                    Event::SlintTouchEvent(window_event) => {
                        self.slint_window.dispatch_event(window_event)
                    }
                    Event::StateEvent => todo!(),
                }
            }

            // redraw the entire window (otherwise we get partial redraws which are more complicated to deal with)
            self.slint_window.request_redraw();

            let mut buffer = self.pool.clone().try_get_rc().unwrap();

            let _is_dirty = self.slint_window.draw_if_needed(|renderer| {
                renderer.render(&mut buffer, 172 as _);
            });

            let renderer = SlintFramebuffer {
                buffer: &mut buffer,
                size: Size {
                    width: 172,
                    height: 320,
                },
            };

            if let Err(_) = tx_render.try_send(buffer) {
                panic!("Failed to tx render buffer");
            }

            // for approx 60fps
            Timer::after(Duration::from_millis(16)).await;
        }
    }
}

async fn render_loop<T: DrawTarget>(
    mut target: T,
    rx_render: Box<DynamicReceiver<'_, Vec<Pixel<T::Color>>>>,
) {
    'render_loop: loop {
        let buf = rx_render.receive().await;

        match target.draw_iter(buf.into_iter()) {
            Ok(..) => {}
            Err(_e) => {
                crate::error!("Failed to draw to draw target.");
                break 'render_loop;
            }
        };
    }
}
