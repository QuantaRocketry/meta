use defmt::*;
use embassy_executor::{SpawnError, Spawner};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Channel, mutex::Mutex};
use embassy_time::{Duration, Timer};

use crate::system;

pub struct StateMachine;

#[derive(Default, Format)]
pub struct InertialState {
    altitude: f32,
    attitude: f32,
}

static STATE_MUTEX: Mutex<ThreadModeRawMutex, Option<InertialState>> = Mutex::new(None);
static MEASUREMENT_CHANNEL: Channel<ThreadModeRawMutex, crate::system::SensorMeasurement, 10> =
    Channel::new();

pub async fn start(spawner: &Spawner) -> Result<(), SpawnError> {
    let state = InertialState::default();
    *(STATE_MUTEX.lock().await) = Some(state);
    spawner.spawn(state_machine_task())
}

pub async fn update(measurement: system::SensorMeasurement) {
    MEASUREMENT_CHANNEL.send(measurement).await;
}

#[embassy_executor::task(pool_size = 1)]
async fn state_machine_task() {
    let event_receiver = MEASUREMENT_CHANNEL.receiver();

    loop {
        Timer::after_secs(1).await;

        let event = event_receiver.try_receive();
        if let Ok(event) = event {
            info!("{:?}", event);
        } else {
            let mut state_guard = STATE_MUTEX.lock().await;
            let state = state_guard.as_mut().unwrap();
            info!("{:?}", state);
        }
    }
}
