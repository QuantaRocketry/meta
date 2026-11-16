use crate::device::hardware;
use crate::module::gnss::GnssStateManager;
use crate::module::*;
use crate::{error, info, warn};
use embassy_executor::Spawner;
use embassy_futures::select::select_array;
use embassy_nrf::{gpio, spim};
use embassy_sync::blocking_mutex::raw::RawMutex;
use embassy_sync::{
    blocking_mutex::raw::{NoopRawMutex, ThreadModeRawMutex},
    signal::Signal,
    watch::{self, Watch},
    zerocopy_channel,
};
mod config;
use common::module::battery::BatteryState;
use common::module::radio::{self, RadioResources};
pub use config::{ConfigStorage, SystemConfig};
use embassy_time::{Duration, Ticker, Timer};
use embedded_hal_async::spi::SpiDevice;
use embedded_hal_bus::spi::ExclusiveDevice;
use qcp::RadioSettings;

pub type Mutex = ThreadModeRawMutex;

pub struct SystemState {
    gnss_config: Watch<Mutex, gnss::GnssState, 2>,
    battery: Watch<Mutex, BatteryState, 2>,
    pub config: &'static SystemConfig,

    #[cfg(feature = "has-compass")]
    compass: Watch<Mutex, compass::CompassState, 1>,
}

impl SystemState {
    pub fn new(config: &'static SystemConfig) -> Self {
        let gnss_config = Watch::new();
        let battery = Watch::new();

        #[cfg(feature = "has-compass")]
        let compass = Watch::new();

        Self {
            gnss_config,
            battery,
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

impl SystemState {
    pub fn battery_sender(&self) -> watch::DynSender<'_, BatteryState> {
        self.battery.dyn_sender()
    }

    pub fn try_get_battery_state_watcher(&self) -> Option<watch::DynReceiver<'_, BatteryState>> {
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

impl
    radio::RadioResources<
        embedded_hal_bus::spi::ExclusiveDevice<
            embassy_nrf::spim::Spim<'static>,
            embassy_nrf::gpio::Output<'static>,
            embassy_time::Delay,
        >,
        embassy_nrf::gpio::Input<'static>,
        Mutex,
    > for &SystemState
{
    fn try_get_resources(
        &self,
    ) -> Result<
        (
            embedded_hal_bus::spi::ExclusiveDevice<
                spim::Spim<'static>,
                gpio::Output<'static>,
                embassy_time::Delay,
            >,
            gpio::Input<'static>,
            zerocopy_channel::Receiver<'static, Mutex, radio::RadioBuffer>,
        ),
        (),
    > {
        // let mut config = spim::Config::default();
        // config.frequency = spim::Frequency::M4; // Set clock speed
        // config.mode = spim::MODE_0; // CPOL=0, CPHA=0
        // config.orc = 0x00; // Over-read character

        // let nss = gpio::Output::new(
        //     self.radio_resources.cs,
        //     gpio::Level::High,
        //     gpio::OutputDrive::Standard,
        // );
        // let reset = gpio::Output::new(
        //     self.radio_resources.reset,
        //     gpio::Level::High,
        //     gpio::OutputDrive::Standard,
        // );
        // let mut dio1 = gpio::Input::new(self.radio_resources.dio1, gpio::Pull::Down);
        // let busy = gpio::Input::new(self.radio_resources.busy, gpio::Pull::None);
        // let rf_switch_rx = gpio::Output::new(
        //     self.radio_resources.sw,
        //     gpio::Level::Low,
        //     gpio::OutputDrive::Standard,
        // );

        // let spim = spim::Spim::new(
        //     self.radio_resources.spi,
        //     hardware::Irqs,
        //     self.radio_resources.sck,
        //     self.radio_resources.miso,
        //     self.radio_resources.mosi,
        //     config,
        // );
        // let spi_device = ExclusiveDevice::new(spim, nss, embassy_time::Delay);
        // let receiver = match self.radio_config.dyn_receiver() {
        //     Some(r) => r,
        //     None => return Err(()),
        // };
        // return Ok((spi_device, dio1, receiver));
        todo!()
    }
}

#[embassy_executor::task]
pub async fn runner(
    _spawner: Spawner,
    state: &'static crate::system::SystemState,
    mut radio_tx: zerocopy_channel::Sender<'static, Mutex, radio::RadioBuffer>,
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
    radio_tx: &mut zerocopy_channel::Sender<'static, Mutex, radio::RadioBuffer>,
    _state: Option<gnss::GnssState>,
) {
    let tx_buf = radio_tx.send().await;
    tx_buf.clear();
    if let Err(e) = tx_buf.extend_from_slice(b"hello") {
        error!("{}", e);
    }

    radio_tx.send_done();
}
