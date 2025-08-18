use defmt::*;
use embassy_executor::{SpawnError, Spawner};
use embassy_rp::gpio::{Level, Output};
use embassy_sync::channel::Channel;
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex, mutex::MutexGuard};

pub mod indicator;
pub mod interface;
pub mod state;
pub mod state_machine;

use crate::system;
use crate::system::state_machine::StateMachine;
use crate::{
    resources::AssignedResources,
    system::{indicator::IndicatorTrait, interface::USBInterface},
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

#[derive(Default, Copy, Clone, Format)]
pub struct System;

static SYSTEM_MUTEX: Mutex<ThreadModeRawMutex, Option<System>> = Mutex::new(None);
static SYSTEM_CHANNEL: Channel<ThreadModeRawMutex, Event, 10> = Channel::new();

impl System {}

pub async fn start(spawner: &Spawner) -> Result<(), SpawnError> {
    let system = System;
    *(SYSTEM_MUTEX.lock().await) = Some(system);
    spawner.spawn(system_task())?;

    Ok(())
}

pub async fn send_event(event: Event) {
    SYSTEM_CHANNEL.send(event).await;
}

#[embassy_executor::task(pool_size = 1)]
async fn system_task() {
    let event_receiver = SYSTEM_CHANNEL.receiver();

    loop {
        // receive the events, halting the task until an event is received
        let event = event_receiver.receive().await;

        match event {
            Event::Measurement(sensor_measurement) => {
                state_machine::update(sensor_measurement).await
            }
            Event::StateUpdate(_) => system::indicator::indicate_event(event).await,
        }
    }
}
