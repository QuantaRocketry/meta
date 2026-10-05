use embassy_sync::watch::DynSender;
use embassy_time::{Duration, Ticker};
use heapless::HistoryBuf;

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BatteryState {
    pub voltage_mv: u16,
}

/// Hardware access to a battery voltage measurement.
pub trait Battery {
    /// Measure the current battery voltage in millivolts.
    async fn sample_mv(&mut self) -> u16;
}

pub async fn runner(mut battery: impl Battery, sender: DynSender<'_, BatteryState>) -> ! {
    const SAMPLE_PERIOD: Duration = Duration::from_hz(5);
    const FILTER_LEN: usize = 20;

    let mut ticker = Ticker::every(SAMPLE_PERIOD);
    let mut battery_state = BatteryState::default();
    let mut history = HistoryBuf::<u16, FILTER_LEN>::new();
    loop {
        ticker.next().await;

        history.write(battery.sample_mv().await);
        let sum: u32 = history.iter().map(|&v| u32::from(v)).sum();
        let voltage_mv = (sum / history.len() as u32) as u16;
        if voltage_mv != battery_state.voltage_mv {
            battery_state.voltage_mv = voltage_mv;
            sender.send(battery_state);
        }
    }
}
