//! Reads the magnetometer at 10 Hz on a Linux system with an I2C bus
//! (e.g. a Raspberry Pi's `/dev/i2c-1`).
//!
//! Run with:
//!   cargo run --example native -- /dev/i2c-1
//!
//! Wire SDO/SA1 to GND for `SlaveAddress::Low` (0x1C), or to Vdd_IO for
//! `SlaveAddress::High` (0x1E) - see datasheet Section 5.1.1.

use std::{thread, time::Duration};

use linux_embedded_hal::I2cdev;

use lis3mdl::{FullScale, Lis3mdl, SlaveAddress};

fn main() {
    let bus_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/dev/i2c-1".into());
    let i2c = I2cdev::new(&bus_path).expect("failed to open I2C bus");

    let mut mag = Lis3mdl::try_new(i2c, SlaveAddress::Low, FullScale::Gauss4)
        .expect("failed to initialize LIS3MDL");

    println!("WHO_AM_I = 0x{:02X}", mag.who_am_i().unwrap());

    loop {
        if mag.data_ready().unwrap() {
            let field = mag.read_magnetometer_gauss().unwrap();
            let temp_c = mag.read_temperature_celsius().unwrap();
            println!(
                "x={:>8.4} G  y={:>8.4} G  z={:>8.4} G  temp={:.1} C",
                field.x, field.y, field.z, temp_c
            );
        }
        thread::sleep(Duration::from_millis(100));
    }
}
