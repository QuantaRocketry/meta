# Wio L1

This is based on the Seeed Wio L1 Tracker Pro

## Building

This project uses the UF2 bootloader that comes pre-installed on the device. Run `cargo uf2` to build the UF2 firmware. Upload the firmware to the device by following the instructions on the Seeed documentation.

## Debugging

This project uses `cargo-embed` and `defmt` for debugging. Run `cargo run --release` to build, flash, and view the RTT logs from the device in release mode.
