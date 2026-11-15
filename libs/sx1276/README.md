# sx1276

`no_std` driver for the Semtech SX1276/77/78/79 (HopeRF RFM95/96/97/98) in
**FSK/OOK packet mode**, receive side. Built on [`device-driver`](https://crates.io/crates/device-driver).

* `registers.rs`: the register map (DSL): bitfields, 16/24-bit big-endian values,
  the repeated sync-word register and the FIFO as a `buffer`.
* `lib.rs`: `SpiInterface` (any `embedded_hal::spi::SpiDevice`) and `Sx1276`
  (reset, configure, receive).

## Usage

FSK/OOK receive (`Sx1276`) and LoRa (`lora::Lora`) are separate drivers over the same
SPI device and reset pin. `release()` hands those back so you can switch modulation.
Both need an `embedded_hal::spi::SpiDevice` that holds chip select for each transaction
(e.g. `embedded_hal_bus::spi::ExclusiveDevice`).

```rust
use sx1276::lora::{Config, Lora};

let mut radio = Lora::new(spi_device, reset_pin);
radio.init(&mut delay, &Config::new(915_000_000))?;   // 125 kHz, SF7, 4/5, CRC on
radio.start_rx_continuous()?;                         // DIO0 rises on RxDone
let mut buf = [0u8; 255];
if let Some(p) = radio.read_packet(&mut buf)? {       // Err(PayloadCrc) if corrupt
    let payload = &buf[..p.len];                      // p.rssi_dbm, p.snr_quarter_db
}
radio.transmit(&mut delay, b"ping", 2_000)?;          // blocks until TxDone
```

LoRa also has `start_rx_single`, `set_rx_timeout_symbols`, `start_transmit`/`tx_done`,
channel activity detection (`start_cad`/`cad_result`), explicit/implicit headers,
SF6..SF12, 7.8..500 kHz, automatic low-data-rate optimisation, and PA_BOOST/RFO/+20 dBm
power selection. Not covered: frequency hopping, the 500 kHz sensitivity workaround from
Semtech's errata (those registers are not in the datasheet), and the 169 MHz band's
RSSI calibration.

The register maps (`registers` for FSK/OOK, `lora_registers` for LoRa) were checked against
the Semtech datasheet Rev. 4 (March 2015), sections 6.2 and 6.4. The FSK path has been
exercised on real hardware via `feather-rp2040-rfm95`; **the LoRa driver has only been run
against the simulated chip in `tests/`, not on hardware.**

Note `docs/SX1276.PDF` is only the 1-page product brief; the register tables are in
the full datasheet (`DS_SX1276-7-8-9`, 132 pages).

`cargo test` runs both drivers against a simulated register file.
