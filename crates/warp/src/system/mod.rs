use defmt::*;
use embassy_executor::{SpawnError, Spawner};
use embassy_rp::gpio::{Level, Output};
use embassy_sync::channel::Channel;
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex, mutex::MutexGuard};
use embassy_time::{Duration, Timer};
use embedded_hal::digital::StatefulOutputPin;

pub mod indicator;
pub mod interface;
pub mod state;
pub mod state_machine;

use crate::system;
use crate::system::indicator::LEDIndicator;
use crate::system::state_machine::StateMachine;
use crate::{
    resources::AssignedResources,
    system::{indicator::IndicatorTrait},
};

#[derive(Copy, Clone, Format)]
pub enum Error {
    Alloc,
}

#[derive(Copy, Clone, Format)]
pub enum State {
    Initializing,
    Okay,
    Error(Error),
}

#[derive(Copy, Clone, Format)]
pub enum Event {
    Measurement(SensorMeasurement),
    StateUpdate(State),
}

#[derive(Copy, Clone, Format)]
pub enum SensorMeasurement {
    Accel(XYZMeasurement),
    Gyro(XYZMeasurement),
    Pressure(f32),
    Temperature(f32),
}

#[derive(Default, Copy, Clone, Format)]
pub struct XYZMeasurement {
    x: f32,
    y: f32,
    z: f32,
}

pub struct System {
    indicator_notifier: &'static indicator::IndicatorNotifier,
}

impl System {
    pub fn new(indicator_notifier: &'static indicator::IndicatorNotifier) -> Self {
        Self { indicator_notifier }
    }

    pub async fn run(&mut self) -> ! {
        let mut temp_state = system::State::Initializing;
        loop {
            Timer::after(Duration::from_millis(5000)).await;

            match temp_state {
                State::Initializing => temp_state = State::Okay,
                State::Okay => temp_state = State::Error(Error::Alloc),
                State::Error(_) => temp_state = State::Initializing,
            }
            self.indicator_notifier.signal(temp_state);
        }
    }
}
