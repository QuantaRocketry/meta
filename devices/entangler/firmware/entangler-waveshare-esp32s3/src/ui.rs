use alloc::{rc::Rc, vec::Vec};

use embassy_executor::{Executor, Spawner};
use embassy_sync::{
    channel::{DynamicReceiver, DynamicSender, Sender},
    watch::DynAnonReceiver,
};
use embassy_time::{Duration, Instant, Timer};
// use entangler_common::ui::MainWindow;
use log::*;
// use object_pool::{Pool, Reusable};
use slint::{
    ComponentHandle, PlatformError,
    platform::{
        PointerEventButton, WindowAdapter, WindowEvent,
        software_renderer::{MinimalSoftwareWindow, RepaintBufferType},
    },
};
use static_cell::StaticCell;

pub struct EmbassyBackend {
    window: Rc<MinimalSoftwareWindow>,
}

impl EmbassyBackend {
    pub fn new(window: Rc<MinimalSoftwareWindow>) -> Self {
        Self { window }
    }
}

impl slint::platform::Platform for EmbassyBackend {
    fn create_window_adapter(
        &self,
    ) -> Result<Rc<dyn WindowAdapter>, slint::platform::PlatformError> {
        let window = self.window.clone();
        crate::info!("create_window_adapter called");
        Ok(window)
    }

    fn duration_since_start(&self) -> core::time::Duration {
        Instant::now().duration_since(Instant::from_secs(0)).into()
    }
}

#[embassy_executor::task]
async fn draw_task(
    window: Rc<MinimalSoftwareWindow>,
    tx_render: DynamicSender<'static, Reusable<'static, Vec<crate::display::ColorFormat>>>,
    rx_event: DynamicReceiver<'static, WindowEvent>,
    pool: &'static Pool<Vec<crate::display::ColorFormat>>,
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
                renderer.render(&mut buffer, crate::display::DISPLAY_WIDTH as _);
                tx_render.send(buffer).ok();
            }
            None => {
                // this happens when the MainWindow hasn't yet been created or if it has been closed by the user
            }
        });

        // for approx 60fps
        Timer::after(Duration::from_millis(16)).await;
    }
}
