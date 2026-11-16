use core::time::Duration;

use embassy_executor::Spawner;
use embassy_nrf::pwm::{self, SimplePwm};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::Timer;

use crate::device::hardware::BuzzerResources;
use crate::info;

#[embassy_executor::task]
pub async fn runner(_spawner: Spawner, r: BuzzerResources) {
    info!("Buzzer task started");

    let config = pwm::SimpleConfig::default();
    let mut pwm = SimplePwm::new_1ch(r.pwm, r.pin, &config);
    pwm.enable();
    pwm.set_prescaler(pwm::Prescaler::Div1);
    pwm.set_max_duty(32767);
    pwm.set_duty(0, pwm::DutyCycle::normal(0));
    loop {
        beep(&mut pwm, Duration::from_millis(100)).await;
        Timer::after_millis(4900).await;
    }
}

async fn beep(pwm: &mut SimplePwm<'_>, duration: Duration) {
    pwm.set_duty(0, pwm::DutyCycle::normal(pwm.max_duty() / 2));
    Timer::after_millis(duration.as_millis() as u64).await;
    pwm.set_duty(0, pwm::DutyCycle::normal(0));
}
