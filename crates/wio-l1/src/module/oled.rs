use core::fmt::Write;

use buoyant::app::Harness as _;
use buoyant::event::Event;
use buoyant::render_target::{EmbeddedGraphicsRenderTarget, RenderTarget as _};
use display_interface_i2c::I2CInterface;
use embassy_executor::Spawner;
use embassy_nrf::twim::{self, Frequency};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Instant, Ticker, Timer};
use embedded_graphics::geometry::OriginDimensions as _;
use embedded_graphics::pixelcolor::BinaryColor;
use interface;
use oled_async::displayrotation::DisplayRotation;
use static_cell::ConstStaticCell;

use crate::module::gnss::GnssStateManager;
use crate::{device::hardware::Irqs, device::hardware::OledResources};
use crate::{error, info};

#[derive(Debug, Clone, PartialEq)]
pub enum OledEvent {
    Navigation(Event),
}

#[embassy_executor::task]
pub async fn runner(
    _spawner: Spawner,
    r: OledResources,
    state: &'static crate::system::SystemState,
) {
    let mut display = {
        static RAM_BUFFER: ConstStaticCell<[u8; 16]> = ConstStaticCell::new([0; 16]);
        let mut config = twim::Config::default();
        config.frequency = Frequency::K400;
        let mut i2c = twim::Twim::new(r.i2c, Irqs, r.sda, r.scl, config, RAM_BUFFER.take());
        Timer::after_millis(1).await;

        // Probe for the OLED address (standard addresses are 0x3C and 0x3D)
        let mut buf = [0u8];
        let addr = {
            match i2c.read(0x3c, &mut buf).await {
                Ok(_) => 0x3c,
                Err(twim::Error::AddressNack) => 0x3d,
                Err(e) => {
                    error!("I2C error {:?}", e);
                    return;
                }
            }
        };

        Timer::after_millis(1).await;

        if let Err(e) = i2c.read(addr, &mut buf).await {
            error!("oled error 0x{:x} {:?}", addr, e);
            return;
        }

        let i2c_interface = I2CInterface::new(i2c, addr, 0x40);

        let raw_disp = oled_async::Builder::new(oled_async::displays::sh1106::Sh1106_128_64 {})
            .with_rotation(DisplayRotation::Rotate0)
            .connect(i2c_interface);

        let mut display: oled_async::mode::GraphicsMode<_, _> = raw_disp.into();

        if let Err(e) = display.init().await {
            error! {"init {:?}", e};
            return;
        };
        display.clear();
        if let Err(e) = display.flush().await {
            error! {"flush {:?}", e};
            return;
        }
        info!("Display Initialized");
        display
    };

    let size = display.size().into();
    let mut app = buoyant::app::App::new(interface::State::default(), size, interface::view);
    let mut target = EmbeddedGraphicsRenderTarget::new_hinted(&mut display, BinaryColor::Off);

    let mut frame_ticker = Ticker::every(Duration::from_hz(30));

    let mut battery_watcher = state
        .try_get_battery_state_watcher()
        .expect("failed to get battery watcher");

    let mut gnss_watcher = state
        .try_get_gnss_state_watcher()
        .expect("failed to get gnss watcher");
    loop {
        // limit polling for updates
        frame_ticker.next().await;

        // Sync app time with real wall clock time
        app.set_time(core::time::Duration::from_millis(
            Instant::now().as_millis(),
        ));

        // apply events
        {
            let receiver = crate::module::joystick::JOYSTICK_CHANNEL.receiver();
            while receiver.len() > 0 {
                if let Ok(event) = receiver.try_receive() {
                    app.send(event);
                }
            }

            if let Some(b_state) = battery_watcher.try_changed() {
                const BATTERY_LOW_MV: u16 = 3400;
                const BATTERY_HIGH_MV: u16 = 4200;
                let mv = b_state.voltage_mv as f32;

                let percent = ((mv - BATTERY_LOW_MV as f32)
                    / (BATTERY_HIGH_MV - BATTERY_LOW_MV) as f32)
                    .clamp(0.0, 1.0);
                app.state_mut().battery_percentage = percent;
            }

            if let Some(gnss_state) = gnss_watcher.try_changed() {
                let mut app_state = app.state_mut();
                app_state.location.coordinate.latitude = gnss_state.latitude;
                app_state.location.coordinate.longitude = gnss_state.longitude;
                app_state.location.coordinate.altitude = gnss_state.altitude;

                // for testing: surface our own GNSS fix as a selectable POI
                const GROUND_ID: &str = "ID-GROUND";
                if let Some(ground) = app_state
                    .visible_pois
                    .iter_mut()
                    .find(|poi| poi.id == GROUND_ID)
                {
                    ground.coordinate.latitude = gnss_state.latitude;
                    ground.coordinate.longitude = gnss_state.longitude;
                    ground.coordinate.altitude = gnss_state.altitude;
                } else {
                    let mut ground = interface::TrackedPOI::default();
                    ground.id.push_str(GROUND_ID).unwrap();
                    ground.coordinate.latitude = gnss_state.latitude;
                    ground.coordinate.longitude = gnss_state.longitude;
                    ground.coordinate.altitude = gnss_state.altitude;
                    let _ = app_state.visible_pois.push(ground);
                }
                if app_state.poi_id.is_empty() {
                    app_state.poi_id.push_str(GROUND_ID).unwrap();
                }
            }

            // Update state after timeout
            {
                app.state_mut().heading =
                    (Instant::now().as_millis() as f32 / 1000.0 / 2.0) % 1.0 * 360.0;
            }
        }

        // Only render if active animation was reported or redraw needed
        if app.should_redraw() || target.clear_animation_status() {
            // Render animated transition between source and target trees
            app.render_animated(&mut target, &BinaryColor::On);

            // Draw focus overlay
            // app.draw_focus_overlay(&mut target, BinaryColor::On, 1);

            // Send to the display
            if let Err(e) = target.display_mut().flush().await {
                error!("{:?}", e);
            };

            // Clear for the next frame
            target.clear(BinaryColor::Off);
        }
    }
}
