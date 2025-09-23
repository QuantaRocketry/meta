use defmt::*;
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Receiver};
use embedded_io_async::Write;

use crate::system::Event;
pub struct Interface<W: Write, const N: usize> {
    writer: W,
    channel: Receiver<'static, ThreadModeRawMutex, Event, N>,
}

impl<W: Write, const N: usize> Interface<W, N> {
    pub fn new(writer: W, channel: Receiver<'static, ThreadModeRawMutex, Event, N>) -> Self {
        Self { writer, channel }
    }

    pub async fn run(&mut self) -> ! {
        loop {
            let event = self.channel.receive().await;
            info!("Event rx: {:?}", event);
        }
    }
}
