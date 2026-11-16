use buoyant::event;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Input, Pull};
use embassy_sync::{
    blocking_mutex::raw::ThreadModeRawMutex,
    channel::{Channel, DynamicSender},
};
use embassy_time::{Duration, Timer};

use crate::{debug, device::hardware::JoystickResources, info, warn};

pub static JOYSTICK_CHANNEL: Channel<ThreadModeRawMutex, event::Event, 8> = Channel::new();

#[embassy_executor::task]
pub async fn runner(spawner: Spawner, r: JoystickResources) {
    let up = Input::new(r.up, Pull::None);
    let down = Input::new(r.down, Pull::None);
    let left = Input::new(r.left, Pull::None);
    let right = Input::new(r.right, Pull::None);
    let select = Input::new(r.select, Pull::None);
    let menu = Input::new(r.menu, Pull::None);

    spawner.spawn(button_task(event::Key::UpArrow, up, JOYSTICK_CHANNEL.dyn_sender()).unwrap());
    spawner.spawn(button_task(event::Key::DownArrow, down, JOYSTICK_CHANNEL.dyn_sender()).unwrap());
    spawner.spawn(button_task(event::Key::LeftArrow, left, JOYSTICK_CHANNEL.dyn_sender()).unwrap());
    spawner
        .spawn(button_task(event::Key::RightArrow, right, JOYSTICK_CHANNEL.dyn_sender()).unwrap());
    spawner.spawn(
        button_task(
            event::Key::Character('\n'),
            select,
            JOYSTICK_CHANNEL.dyn_sender(),
        )
        .unwrap(),
    );
    spawner.spawn(button_task(event::Key::Escape, menu, JOYSTICK_CHANNEL.dyn_sender()).unwrap());
}

#[embassy_executor::task(pool_size = 6)]
async fn button_task(
    key: event::Key,
    mut pin: Input<'static>,
    sender: DynamicSender<'static, event::Event>,
) {
    const DEBOUNCE: Duration = Duration::from_millis(20);

    loop {
        // Debounce
        Timer::after(DEBOUNCE).await;
        pin.wait_for_low().await;
        if let Err(embassy_sync::channel::TrySendError::Full(event)) =
            sender.try_send(event::Event::KeyDown(key))
        {
            // warn!("Queue full: {:?}", event);
        };
        // debug!("Button {:?} pressed!", key);
        debug!("Button pressed!");

        // Debounce
        Timer::after(DEBOUNCE).await;
        pin.wait_for_high().await;
        if let Err(embassy_sync::channel::TrySendError::Full(event)) =
            sender.try_send(event::Event::KeyUp(key))
        {
            // warn!("Queue full: {:?}", event);
        };
        // debug!("Button {:?} released!", key);
        debug!("Button released!");
    }
}
