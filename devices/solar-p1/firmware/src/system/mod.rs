use crate::module::gnss::GnssStateManager;
use crate::module::*;
use crate::{error, info, warn};
use embassy_executor::Spawner;
use embassy_futures::select::select_array;
use embassy_sync::{
    blocking_mutex::raw::{NoopRawMutex, ThreadModeRawMutex},
    mutex::Mutex,
    signal::Signal,
    watch::{self, Watch},
    zerocopy_channel,
};

mod config;
pub use config::SystemConfig;
use embassy_time::{Duration, Ticker, Timer};

pub struct SystemState {
    gnss_config: Watch<ThreadModeRawMutex, gnss::GnssState, 1>,
    battery: Watch<ThreadModeRawMutex, battery::BatteryState, 1>,
    radio_config: Watch<ThreadModeRawMutex, radio::RadioConfig, 1>,
    config: &'static SystemConfig,

    #[cfg(feature = "has-compass")]
    compass: Watch<ThreadModeRawMutex, compass::CompassState, 1>,
}

impl SystemState {
    pub fn new(config: &'static SystemConfig) -> Self {
        let gnss_config = Watch::new();
        let battery = Watch::new();
        let radio_config = Watch::new();

        #[cfg(feature = "has-compass")]
        let compass = Watch::new();

        Self {
            gnss_config,
            battery,
            radio_config,
            config,

            #[cfg(feature = "has-compass")]
            compass,
        }
    }
}

impl<'a> gnss::GnssStateManager<'a> for SystemState {
    fn set_gnss_state(&self, config: &gnss::GnssState) {
        self.gnss_config.sender().send(*config);
    }

    fn try_get_gnss_state_watcher(&'a self) -> Option<watch::DynReceiver<'a, gnss::GnssState>> {
        self.gnss_config.dyn_receiver()
    }
}

impl<'a> battery::BatteryStateManager<'a, ThreadModeRawMutex> for SystemState {
    async fn set_battery_state(&self, state: &battery::BatteryState) {
        self.battery.sender().send(*state);
    }

    async fn try_get_battery_state(&self) -> Option<battery::BatteryState> {
        self.battery.try_get()
    }

    fn try_get_battery_state_watcher(
        &'a self,
    ) -> Option<watch::DynReceiver<'a, battery::BatteryState>> {
        self.battery.dyn_receiver()
    }
}

#[cfg(feature = "has-compass")]
impl<'a> compass::CompassStateManager<'a> for SystemState {
    fn set_compass_state(&self, config: &compass::CompassState) {
        self.compass.sender().send(*config);
    }

    fn try_get_compass_state_watcher(
        &'a self,
    ) -> Option<watch::DynReceiver<'a, compass::CompassState>> {
        self.compass.dyn_receiver()
    }
}

impl<'a> radio::RadioConfigManager<'a> for SystemState {
    fn set_radio_config(&self, config: &radio::RadioConfig) {
        self.radio_config.sender().send(*config);
    }

    fn try_get_radio_config_watcher(
        &'a self,
    ) -> Option<embassy_sync::watch::DynReceiver<'a, radio::RadioConfig>> {
        self.radio_config.dyn_receiver()
    }
}

#[embassy_executor::task]
pub async fn runner(
    _spawner: Spawner,
    state: &'static crate::system::SystemState,
    mut radio_tx: zerocopy_channel::Sender<'static, NoopRawMutex, radio::RadioBuffer>,
) {
    info!("started system runner");
    let mut position_watch = {
        match state.try_get_gnss_state_watcher() {
            Some(w) => w,
            None => {
                warn!("system ran out of gnss watchers");
                loop {
                    Timer::after_secs(1).await;
                }
            }
        }
    };

    let mut radio_tx_ticker = Ticker::every(Duration::from_secs(10));
    loop {
        match select_array([radio_tx_ticker.next()]).await {
            (_, 0) => radio_send(&mut radio_tx, position_watch.try_get()).await,
            _ => {}
        };
    }
}

async fn radio_send(
    radio_tx: &mut zerocopy_channel::Sender<'static, NoopRawMutex, radio::RadioBuffer>,
    _state: Option<gnss::GnssState>,
) {
    let tx_buf = radio_tx.send().await;
    tx_buf.clear();
    if let Err(e) = tx_buf.extend_from_slice(b"hello") {
        error!("{}", e);
    }

    radio_tx.send_done();
}
