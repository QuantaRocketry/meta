use alloc::string::String;
use embassy_executor::Spawner;
use embassy_nrf::gpio;
use embassy_nrf::saadc::{self, Saadc};
use embassy_sync::blocking_mutex::raw::{RawMutex, ThreadModeRawMutex};
use embassy_sync::watch::Watch;
use embassy_time::{Duration, Instant, Ticker, Timer, with_timeout};

use crate::{debug, error, info};
use crate::{device::hardware::BatteryResources, device::hardware::Irqs};

const VREF_MV: i32 = 3600; // 0.6V ref × gain factor of 6, in millivolts
const ADC_MAX: i32 = 1 << 12; // 2^12
const DIVIDER: i32 = 2;

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct BatteryState {
    pub voltage_mv: u16,
}

pub trait BatteryStateManager<'a, M: RawMutex> {
    async fn set_battery_state(&self, state: &BatteryState);
    async fn try_get_battery_state(&self) -> Option<BatteryState>;
    fn try_get_battery_state_watcher(
        &'a self,
    ) -> Option<embassy_sync::watch::DynReceiver<'a, BatteryState>>;
}

#[embassy_executor::task]
pub async fn runner(
    _spawner: Spawner,
    r: BatteryResources,
    system_state: &'static crate::system::SystemState,
) {
    // pull ctrl pin high to enable BMS
    let _ctrl_pin = gpio::Output::new(r.ctrl, gpio::Level::High, gpio::OutputDrive::Standard);
    Timer::after_millis(1).await;

    let mut config = saadc::Config::default();
    config.resolution = saadc::Resolution::_8BIT;
    config.oversample = saadc::Oversample::BYPASS;

    let mut channel_config = saadc::ChannelConfig::single_ended(r.adc_read);
    channel_config.gain = saadc::Gain::GAIN1_6;
    channel_config.reference = saadc::Reference::INTERNAL;
    channel_config.time = saadc::Time::_40US; // long acquisition for RC filter impedance

    let mut saadc = Saadc::new(r.saadc, Irqs, config, [channel_config]);
    saadc.calibrate().await;

    let mut ticker = Ticker::every(Duration::from_hz(5));
    let mut battery_state = BatteryState::default();
    loop {
        ticker.next().await;

        let mut buf = [0i16; 1];
        saadc.sample(&mut buf).await;

        let sample = buf[0] as i32;

        // Convert to millivolts at the ADC pin
        let v_pin_mv = (sample * VREF_MV) / ADC_MAX;

        // Multiply by divider ratio to get actual battery voltage
        let v_bat_mv = (v_pin_mv * DIVIDER) as u16;

        if v_bat_mv != battery_state.voltage_mv {
            battery_state.voltage_mv = v_bat_mv;
            system_state.set_battery_state(&battery_state).await;

            // debug!(
            //     "raw={} pin={}mV bat={}mV ({}.{}V)",
            //     sample,
            //     v_pin_mv,
            //     v_bat_mv,
            //     v_bat_mv / 1000,
            //     (v_bat_mv % 1000) / 10
            // );
        }
    }
}
