use cortex_m::prelude::_embedded_hal_digital_OutputPin;
use defmt::*;
use embassy_executor::{SpawnError, Spawner};
use embassy_futures::select::{select, Either};
use embassy_rp::{
    gpio::{AnyPin, Output},
    pio::Pio,
    pio_programs::pwm::{PioPwm, PioPwmProgram},
    pwm::{self, Pwm, SetDutyCycle},
};
use embassy_sync::{
    blocking_mutex::raw::{CriticalSectionRawMutex, ThreadModeRawMutex},
    channel::Channel,
    mutex::Mutex,
    signal::Signal,
};
use embassy_time::{Duration, Timer};
use embedded_hal::digital::StatefulOutputPin;
use static_cell::StaticCell;

use crate::{
    resources::{IndicatorResources, Irqs},
    system::{self, Event},
};

pub trait IndicatorTrait {
    fn set_state(&self, state: system::State);
}

pub type IndicatorNotifier = Signal<CriticalSectionRawMutex, system::State>;
pub const fn notifier() -> IndicatorNotifier {
    Signal::new()
}

pub struct LEDIndicator<P: StatefulOutputPin> {
    notifier: &'static IndicatorNotifier,
    pin: P,
}

impl<P: StatefulOutputPin> LEDIndicator<P> {
    pub fn new(pin: P, notifier: &'static IndicatorNotifier) -> Self {
        Self { notifier, pin }
    }

    pub async fn run(&mut self) -> ! {
        let mut freq = Duration::from_hz(1);
        loop {
            if let Either::Second(s) = select(Timer::after(freq), self.notifier.wait()).await {
                match s {
                    system::State::Initializing => freq = Duration::from_hz(2),
                    system::State::Okay => freq = Duration::from_secs(1),
                    system::State::Error(_) => freq = Duration::from_secs(0),
                }
            }

            if let Err(_) = self.pin.toggle() {
                defmt::error!("Failed to toggle pin");
            };
        }
    }

    pub fn indicate_state(&self, state: system::State) {
        self.notifier.signal(state);
    }
}

