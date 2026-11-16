#![cfg_attr(not(test), no_std)]

pub mod registers;

use device_driver::AsyncRegisterInterface;
use device_driver::RegisterInterface;

/// The 7-bit I2C address of the LSM6DS3 depends on how the `SDO/SA1` pin
/// is wired (datasheet Section 5.1.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlaveAddress {
    /// `SDO/SA1` tied to GND -> `0b1101010` (0x6A)
    Low,
    /// `SDO/SA1` tied to `Vdd_IO` -> `0b1101011` (0x6B)
    High,
}

impl SlaveAddress {
    fn bits(self) -> u8 {
        match self {
            SlaveAddress::Low => 0x6A,
            SlaveAddress::High => 0x6B,
        }
    }
}

/// Everything that can go wrong talking to the sensor.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error<E> {
    /// The underlying I2C bus returned an error.
    Bus(E),
    /// `WHO_AM_I` did not read back `0x6A` as required by the datasheet.
    WrongDevice { got: u8 },
    /// A configuration write failed verification on readback.
    WriteVerifyFailed { register_address: u8 },
}

impl<E> From<E> for Error<E> {
    fn from(e: E) -> Self {
        Error::Bus(e)
    }
}

pub struct I2cInterface<I2C> {
    i2c: I2C,
    address: u8,
}

impl<I2C: embedded_hal::i2c::I2c> RegisterInterface for I2cInterface<I2C> {
    type Error = I2C::Error;
    type AddressType = u8;

    fn write_register(
        &mut self,
        address: Self::AddressType,
        size_bits: u32,
        data: &[u8],
    ) -> Result<(), Self::Error> {
        let final_address = if size_bits > 8 {
            address | 0x80
        } else {
            address
        };
        self.i2c.transaction(
            self.address,
            &mut [
                embedded_hal::i2c::Operation::Write(&[final_address]),
                embedded_hal::i2c::Operation::Write(data),
            ],
        )
    }

    fn read_register(
        &mut self,
        address: Self::AddressType,
        size_bits: u32,
        data: &mut [u8],
    ) -> Result<(), Self::Error> {
        let final_address = if size_bits > 8 {
            address | 0x80
        } else {
            address
        };
        self.i2c.write_read(self.address, &[final_address], data)
    }
}

impl<I2C: embedded_hal_async::i2c::I2c> AsyncRegisterInterface for I2cInterface<I2C> {
    type Error = I2C::Error;
    type AddressType = u8;

    async fn write_register(
        &mut self,
        address: Self::AddressType,
        size_bits: u32,
        data: &[u8],
    ) -> Result<(), Self::Error> {
        let final_address = if size_bits > 8 {
            address | 0x80
        } else {
            address
        };

        let mut buffer = [0u8; 16];
        buffer[0] = final_address;

        let mut len = data.len();
        if len > buffer.len() - 1 {
            // not pretty but we shouldn't be writing more than a couple bytes at a time
            len = buffer.len() - 1;
        }

        buffer[1..=len].copy_from_slice(data);

        // Dispatch as a single, uninterrupted write phase
        self.i2c.write(self.address, &buffer[..1 + len]).await
    }

    async fn read_register(
        &mut self,
        address: Self::AddressType,
        size_bits: u32,
        data: &mut [u8],
    ) -> Result<(), Self::Error> {
        let final_address = if size_bits > 8 {
            address | 0x80
        } else {
            address
        };
        self.i2c
            .write_read(self.address, &[final_address], data)
            .await
    }
}

pub struct Lsm6ds3<I2C> {
    device: registers::Lsm6ds3Registers<I2cInterface<I2C>>,
}

impl<I2C> Lsm6ds3<I2C> {
    const WHO_AM_I: u8 = 0x6A;

    /// Escape hatch: direct access to the `device-driver`-generated
    /// register API, for anything this wrapper doesn't cover (e.g.
    /// interrupt threshold / hard-iron offset registers).
    pub fn registers(&mut self) -> &mut registers::Lsm6ds3Registers<I2cInterface<I2C>> {
        &mut self.device
    }
}

impl<I2C: embedded_hal_async::i2c::I2c> Lsm6ds3<I2C> {
    pub async fn try_new_async(i2c: I2C, address: SlaveAddress) -> Result<Self, Error<I2C::Error>> {
        let mut lsm3ds3 = Self {
            device: registers::Lsm6ds3Registers::new(I2cInterface {
                i2c,
                address: address.bits(),
            }),
        };

        lsm3ds3.init_async().await?;

        Ok(lsm3ds3)
    }

    /// Read the `WHO_AM_I` register. The LSM6DS3 always returns `0x6A`
    pub async fn who_am_i_async(&mut self) -> Result<u8, Error<I2C::Error>> {
        Ok(self.device.who_am_i().read_async().await?.value())
    }

    async fn init_async(&mut self) -> Result<(), Error<I2C::Error>> {
        let id = self.who_am_i_async().await?;
        if id != Self::WHO_AM_I {
            return Err(Error::WrongDevice { got: id });
        }

        self.device
            .ctrl_3_c()
            .write_async(|reg| reg.set_sw_reset(true))
            .await?;

        // busy loop until back up
        loop {
            let status = self.device.ctrl_3_c().read_async().await?;
            if !status.sw_reset() {
                break;
            }
        }

        Ok(())
    }

    /// Reads the raw accelerometer data (X, Y, Z).
    pub async fn read_accelerometer_async(&mut self) -> Result<(i16, i16, i16), Error<I2C::Error>> {
        let out = self.device.out_xl().read_async().await?;

        Ok((out.x() as i16, out.y() as i16, out.z() as i16))
    }

    /// Reads the raw gyroscope data (X, Y, Z).
    pub async fn read_gyroscope_async(&mut self) -> Result<(i16, i16, i16), Error<I2C::Error>> {
        let out = self.device.out_g().read_async().await?;

        Ok((out.x() as i16, out.y() as i16, out.z() as i16))
    }
}
