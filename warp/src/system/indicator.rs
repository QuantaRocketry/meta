use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::{
    gpio::Output,
    pio::Pio,
    pio_programs::pwm::{PioPwm, PioPwmProgram},
    pwm::{self, Pwm, SetDutyCycle},
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Channel, mutex::Mutex};
use embassy_time::{Duration, Timer};
use static_cell::StaticCell;

use crate::{
    resources::{IndicatorResources, Irqs},
    system::{self, Event},
};

pub trait IndicatorTrait {
    fn set_state(&mut self, state: system::State);
}

pub struct LEDIndicator {
    pub state: system::State,
}

static LED_INDICATOR_MUTEX: Mutex<ThreadModeRawMutex, Option<LEDIndicator>> = Mutex::new(None);
static LED_INDICATOR_CHANNEL: Channel<ThreadModeRawMutex, Event, 10> = Channel::new();

impl LEDIndicator {
    pub async fn start(spawner: &Spawner, r: IndicatorResources) {
        let led_indicator = Self {
            state: system::State::Initializing,
        };
        *(LED_INDICATOR_MUTEX.lock().await) = Some(led_indicator);

        let _ = spawner.spawn(indicator_task(r));
    }
}

pub async fn indicate_event(event: system::Event) {
    LED_INDICATOR_CHANNEL.send(event).await;
}

#[embassy_executor::task(pool_size = 1)]
async fn indicator_task(r: IndicatorResources) {
    let desired_freq_hz = 1;
    let clock_freq_hz = embassy_rp::clocks::clk_sys_freq();
    let divider = 16u8;
    let period = (clock_freq_hz / (desired_freq_hz * divider as u32)) as u16 - 1;
    
    let mut c = pwm::Config::default();
    c.top = period;
    c.compare_b = divider.into();

    let mut led_pwm = Pwm::new_output_b(r.led_pwm_slice, r.led_pin, c.clone());

    let event_receiver = LED_INDICATOR_CHANNEL.receiver();

    loop {
        let event = event_receiver.receive().await;

        match event {
            Event::StateUpdate(state) => {
                let mut guard = LED_INDICATOR_MUTEX.lock().await;
                let indicator = guard.as_mut().unwrap();
                indicator.state = state;

                match state {
                    system::State::Initializing | system::State::Okay => {
                        info!("got info");
                        if let Err(_) = led_pwm.set_duty_cycle_percent(50) {
                            info!("PWM Error");
                        };
                    }
                    system::State::Error(_) => {}
                }
            }
            _ => (),
        }
    }
}
