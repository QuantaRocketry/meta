use core::cell::RefCell;

use alloc::format;
use embassy_executor::Spawner;
use embassy_nrf::twim;
use embassy_time::{Duration, Instant, Timer};
use serde::de;

use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;

use lis3mdl;
use lsm6ds3;
use static_cell::StaticCell;
use uf_ahrs::{Mahony, MahonyParams};

use crate::{debug, error, info, warn};
use crate::{device::hardware::CompassResources, device::hardware::Irqs};

const COMPASS_HZ: u64 = 50;

#[derive(Default, Debug, Copy, Clone)]
pub struct CompassState {
    pub heading: f32,
}

pub trait CompassStateManager<'a> {
    fn set_compass_state(&self, config: &CompassState);
    fn try_get_compass_state_watcher(
        &'a self,
    ) -> Option<embassy_sync::watch::DynReceiver<'a, CompassState>>;
}

#[embassy_executor::task]
pub async fn runner(_spawner: Spawner, r: CompassResources) {
    info!("Compass task started");

    let mut config = twim::Config::default();
    config.frequency = twim::Frequency::K100;
    Timer::after_secs(3).await;

    static I2C_BUS: StaticCell<Mutex<NoopRawMutex, twim::Twim>> = StaticCell::new();
    let twi = twim::Twim::new(r.i2c, Irqs, r.sda, r.scl, config, &mut []);
    let i2c_bus = Mutex::new(twi);
    let i2c_bus = I2C_BUS.init(i2c_bus);
    // let i2c_bus = RefCell::new(twi);

    let mut mag = match lis3mdl::Lis3mdl::try_new_async(
        I2cDevice::new(i2c_bus),
        lis3mdl::SlaveAddress::Low,
        lis3mdl::FullScale::Gauss16,
    )
    .await
    {
        Ok(device) => device,
        Err(e) => {
            error!("{:?}", e);
            Timer::after_secs(1).await;
            cortex_m::peripheral::SCB::sys_reset();
        }
    };
    debug!("Lis3dml initialized");

    let mut lsm6ds3_device =
        match lsm6ds3::Lsm6ds3::try_new_async(I2cDevice::new(i2c_bus), lsm6ds3::SlaveAddress::Low)
            .await
        {
            Ok(device) => device,
            Err(e) => {
                error!("{:?}", e);
                Timer::after_secs(1).await;
                cortex_m::peripheral::SCB::sys_reset();
            }
        };
    debug!("Lsm6ds3 initialized");

    let params = MahonyParams::default();
    let _ahrs = Mahony::new(core::time::Duration::from_millis(1000 / COMPASS_HZ), params);

    loop {
        // Timer::after_secs(1).await;
        let loop_top = Instant::now();

        if let Ok(res) = mag.data_ready_async().await {
            if res {
                if let Ok(reading) = mag.read_magnetometer_gauss_async().await {
                    debug!("{} {} {}", reading.x, reading.y, reading.z);
                };
            } else {
                debug!("not ready");
                // let status_raw = mag.registers().status_reg().read_async().await.unwrap();
                // debug!("{}", format!("{:?}", status_raw));
            }
        }

        Timer::at(loop_top + Duration::from_hz(COMPASS_HZ)).await;
    }
}
