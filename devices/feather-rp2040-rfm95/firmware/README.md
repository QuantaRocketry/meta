# feather-rp2040-rfm95

[embassy](https://embassy.dev) firmware for the Adafruit Feather RP2040 RFM95. The SX1276 is put in
**FSK (GFSK) packet mode**. Received packets go to the host as qcp `RawPayload` messages over a USB
CDC serial port (COBS-framed postcard). The host chooses what to receive by sending a qcp
`Protocol` (e.g. eggtimer centre frequency + channel) or a GFSK `RadioConfig`.

Tasks: `radio_task` (init with retry, receive, apply configs), `cdc_task` (qcp over the serial
port), `usb_task` (USB device), `heartbeat_task` (LED), `neopixel_task` (status NeoPixel).

## Build / flash

```sh
cd crates/feather-rp2040-rfm95
cargo run --release     # builds and writes target/thumbv6m-none-eabi/release/feather-rp2040-rfm95.uf2
```

Hold **BOOT**, tap **RESET**, release BOOT, then copy the `.uf2` to the `RPI-RP2` drive.
The SX1276 driver is the separate `sx1276` crate (`crates/sx1276`).
Needs `elf2flash` (`cargo binstall elf2flash`) and the `thumbv6m-none-eabi` target.

## Use

Send `Message::Protocol(Protocol::Eggtimer { .. })` or `Message::RadioConfig(..)`; the feather
retunes and acknowledges by echoing the same kind of message back, or answers `Message::Error(..)`
if it cannot receive that waveform. `Request(Protocol)` / `Request(RadioConfig)` read the running
selection (or its resolved waveform) back, and `Request(Uptime)` is answered too. In singularity, use the qcp protocol in the terminal and the
"Eggtimer receiver" card on the Configuration page.

It boots on the default eggtimer config (`eggtimer::Protocol::default()`, 920.985 MHz).
`sx1276::Config` is derived from the waveform in `libs/sx1276/src/qcp.rs` (RxBw is the smallest
filter with 50 % margin over bitrate + 2 x deviation; eggtimer gets 125 kHz).

The TX's CRC-16/XMODEM can't be checked by the SX1276 (hardware CRC is off), so while the chip
cannot check the CRC, packets with any non-ASCII byte are dropped. Payloads are chunks of an NMEA
stream and are forwarded verbatim. The red LED blinks at 1 Hz while the radio runs, 5 Hz if not.
The NeoPixel (GPIO4) flashes blue once per received packet, and three times in a second when a
config from the host is applied.
