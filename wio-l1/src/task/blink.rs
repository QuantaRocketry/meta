use embedded_hal::digital::StatefulOutputPin;
use embedded_hal_async::delay::DelayNs;
use defmt::info;

pub async fn blink_task(mut led: impl StatefulOutputPin, mut d: impl DelayNs) {
    info!("Blink task started");
    loop {
        let _ = led.toggle();
        info!("Blink!");

        d.delay_ms(500_u32).await;
    }
}
