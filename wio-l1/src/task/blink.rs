use embedded_hal::digital::StatefulOutputPin;
use embedded_hal_async::delay::DelayNs;

pub async fn blink_task(mut led: impl StatefulOutputPin, mut d: impl DelayNs) {
    loop {
        let _ = led.toggle();

        d.delay_ms(500_u32).await;
    }
}
