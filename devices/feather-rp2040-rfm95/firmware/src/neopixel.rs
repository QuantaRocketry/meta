//! The status NeoPixel (GPIO4 on the Feather RP2040 RFM95). Other tasks report
//! what happened with [`notify`]; `neopixel_task` turns each event into a
//! pattern of blue flashes.

use embassy_rp::peripherals::PIO0;
use embassy_rp::pio_programs::ws2812::{Grb, PioWs2812};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::Timer;
use smart_leds::RGB8;

/// Something worth showing on the pixel.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Notify {
    /// The radio received a packet.
    PacketReceived,
    /// The radio took a config sent by the host.
    ConfigApplied,
}

/// One flash per colour in `colors`, each lit for `on_ms` then dark for `off_ms`.
struct Pattern {
    colors: &'static [RGB8],
    on_ms: u64,
    off_ms: u64,
}

impl Pattern {
    const fn total_ms(&self) -> u64 {
        self.colors.len() as u64 * (self.on_ms + self.off_ms)
    }
}

const GREEN: RGB8 = RGB8 { r: 0, g: 40, b: 0 };
const BLUE: RGB8 = RGB8 { r: 0, g: 0, b: 40 };
const OFF: RGB8 = RGB8 { r: 0, g: 0, b: 0 };

const PACKET: Pattern = Pattern { colors: &[BLUE], on_ms: 80, off_ms: 0 };
const CONFIG: Pattern = Pattern { colors: &[GREEN, GREEN, GREEN], on_ms: 150, off_ms: 183 };
const _: () = assert!(CONFIG.total_ms() <= 1000, "three flashes fit in a second");

impl Notify {
    fn pattern(self) -> Pattern {
        match self {
            Notify::PacketReceived => PACKET,
            Notify::ConfigApplied => CONFIG,
        }
    }
}

/// Events wait their turn, so a config acknowledgement is not lost behind a
/// packet flash. When it is full the event is dropped, which is fine for a light.
static EVENTS: Channel<CriticalSectionRawMutex, Notify, 4> = Channel::new();

/// Report `event` to the pixel. Never waits.
pub fn notify(event: Notify) {
    let _ = EVENTS.try_send(event);
}

pub type Pixel = PioWs2812<'static, PIO0, 0, 1, Grb>;

#[embassy_executor::task]
pub async fn neopixel_task(mut pixel: Pixel) -> ! {
    pixel.write(&[OFF]).await;
    loop {
        let pattern = EVENTS.receive().await.pattern();
        for (i, color) in pattern.colors.iter().enumerate() {
            pixel.write(&[*color]).await;
            Timer::after_millis(pattern.on_ms).await;
            pixel.write(&[OFF]).await;
            if i + 1 < pattern.colors.len() {
                Timer::after_millis(pattern.off_ms).await;
            }
        }
    }
}
