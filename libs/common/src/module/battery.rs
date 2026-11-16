use embassy_sync::watch::DynSender;
use embassy_time::{Duration, Ticker};
use heapless::HistoryBuf;

use crate::debug;
pub use qcp::{BatteryState, ChargeStatus};

/// Hardware access to a battery voltage measurement.
pub trait Battery {
    /// Measure the current battery charge as a fraction from 0.0 (empty) to 1.0 (full).
    async fn sample(&mut self) -> f32;
    async fn charge_status(&mut self) -> ChargeStatus;
}

pub async fn runner(mut battery: impl Battery, sender: DynSender<'_, BatteryState>) -> ! {
    const SAMPLE_PERIOD: Duration = Duration::from_hz(5);
    const FILTER_LEN: usize = 20;

    let mut sample_ticker = Ticker::every(SAMPLE_PERIOD);
    let mut battery_state = BatteryState::default();
    let mut history = HistoryBuf::<f32, FILTER_LEN>::new();
    loop {
        sample_ticker.next().await;

        history.write(battery.sample().await);
        let sum: f32 = history.iter().map(|&v| f32::from(v)).sum();
        let charge = sum / history.len() as f32;
        let percent = (charge * 100.0 + 0.5).clamp(0.0, 100.0) as u8;
        if percent != battery_state.percent {
            battery_state.percent = percent;
            let status = battery.charge_status().await;
            battery_state.status = status;
            sender.send(battery_state);
        }
    }
}
