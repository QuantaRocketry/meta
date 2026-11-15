# feather-rp2040-rfm95

[embassy](https://embassy.dev) firmware for the Adafruit Feather RP2040 RFM95. The SX1276 is put in
**FSK (GFSK) packet mode** and every packet it receives is written, as ASCII, to a USB
CDC serial port.

Tasks: `radio_task` (init with retry, receive, idle status line), `cdc_task` (feeds the
serial port from a byte pipe), `usb_task` (USB device), `heartbeat_task` (LED).

## Build / flash

```sh
cd crates/feather-rp2040-rfm95
cargo run --release     # builds and writes target/thumbv6m-none-eabi/release/feather-rp2040-rfm95.uf2
```

Hold **BOOT**, tap **RESET**, release BOOT, then copy the `.uf2` to the `RPI-RP2` drive.
The SX1276 driver is the separate `sx1276` crate (`crates/sx1276`).
Needs `elf2flash` (`cargo binstall elf2flash`) and the `thumbv6m-none-eabi` target.

## Use

Open the new serial port (e.g. `picocom /dev/ttyACM0`). One line per packet.
Non-printable bytes are shown as `.`; trailing CR/LF/NUL are trimmed.
The red LED blinks at 1 Hz while the firmware runs.

## Radio settings (`RADIO` in `src/main.rs`)

Matches the Eggtimer/Eggfinder TX decoded by `eggtimer-decoder`: 2-FSK, 9600 baud,
35 kHz deviation, RxBw 125 kHz, preamble `0x55`x5, sync `2D D4`, variable-length packets.
Frequency = centre 925 MHz + channel 4 offset (+985 kHz) = **925.985 MHz** (`CENTER_FREQ_HZ`,
`CHANNEL`; channel offsets follow `eggtimer-decoder`, only channel 0 is measured there).

The TX's CRC-16/XMODEM can't be checked by the SX1276 (hardware CRC is off), so packets with
any non-ASCII byte are dropped. Payloads are chunks of an NMEA stream and are written verbatim,
so sentences reassemble in the output. While idle, a status line is printed every 2 s.
