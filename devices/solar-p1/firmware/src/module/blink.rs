use crate::{debug, device::hardware::LedResources, info};
use embassy_executor::Spawner;
use embassy_nrf::gpio;
use embassy_time::{Duration, Timer};
use embedded_hal::digital::StatefulOutputPin;

#[embassy_executor::task]
pub async fn runner(_spawner: Spawner, r: LedResources) {
    let led_pin = gpio::Output::new(r.heartbeat, gpio::Level::Low, gpio::OutputDrive::Standard);
    hz(led_pin, Duration::from_hz(1)).await;
}

async fn hz(mut led: impl StatefulOutputPin, period: Duration) {
    info!("Blink task started");
    let mut ticker = embassy_time::Ticker::every(period);
    loop {
        let _ = led.set_low();
        ticker.next().await;

        debug!("Blink!");
        let _ = led.set_high();
        Timer::after_millis(100).await;
    }
}
