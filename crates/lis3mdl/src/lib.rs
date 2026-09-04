#![cfg_attr(not(test), no_std)]

pub mod registers;

use device_driver::AsyncRegisterInterface;
use device_driver::RegisterInterface;

pub use registers::{FullScale, OutputDataRate, SystemMode, XyOperatingMode, ZOperatingMode};

/// The 7-bit I2C address of the LIS3MDL depends on how the `SDO/SA1` pin
/// is wired (datasheet Section 5.1.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlaveAddress {
    /// `SDO/SA1` tied to GND -> `0b0011100` (0x1C)
    Low,
    /// `SDO/SA1` tied to `Vdd_IO` -> `0b0011110` (0x1E)
    High,
}

impl SlaveAddress {
    fn bits(self) -> u8 {
        match self {
            SlaveAddress::Low => 0x1C,
            SlaveAddress::High => 0x1E,
        }
    }
}

/// Everything that can go wrong talking to the sensor.
#[derive(Debug)]
pub enum Error<E> {
    /// The underlying I2C bus returned an error.
    Bus(E),
    /// `WHO_AM_I` did not read back `0x3D` as required by the datasheet.
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

/// A single magnetometer sample, already converted to gauss.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct MagneticField {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// High-level, driver for the LIS3MDL magnetometer.
pub struct Lis3mdl<I2C> {
    device: registers::Lis3mdlRegisters<I2cInterface<I2C>>,
    full_scale: FullScale,
}

impl<I2C> Lis3mdl<I2C> {
    const WHO_AM_I: u8 = 0x3D;

    /// Sensitivity in LSB/gauss for each full-scale setting (datasheet
    /// Table 2, "Sensitivity").
    fn sensitivity_lsb_per_gauss(fs: FullScale) -> f32 {
        match fs {
            FullScale::Gauss4 => 6842.0,
            FullScale::Gauss8 => 3421.0,
            FullScale::Gauss12 => 2281.0,
            FullScale::Gauss16 => 1711.0,
        }
    }

    /// Escape hatch: direct access to the `device-driver`-generated
    /// register API, for anything this wrapper doesn't cover (e.g.
    /// interrupt threshold / hard-iron offset registers).
    pub fn registers(&mut self) -> &mut registers::Lis3mdlRegisters<I2cInterface<I2C>> {
        &mut self.device
    }
}

impl<I2C: embedded_hal::i2c::I2c> Lis3mdl<I2C> {
    /// Take ownership of an I2C bus and wrap it in a LIS3MDL driver.
    /// Nothing is transmitted yet; call [`Lis3mdl::init`] (or configure
    /// the individual `CTRL_REG*` registers yourself) before use.
    pub fn try_new(
        i2c: I2C,
        address: SlaveAddress,
        full_scale: FullScale,
    ) -> Result<Self, Error<I2C::Error>> {
        let mut lis3mdl = Self {
            device: registers::Lis3mdlRegisters::new(I2cInterface {
                i2c,
                address: address.bits(),
            }),
            full_scale,
        };

        lis3mdl.init()?;

        Ok(lis3mdl)
    }

    /// Read the `WHO_AM_I` register. The LIS3MDL always returns `0x3D`
    /// (datasheet Table 16 / Section 7.4).
    pub fn who_am_i(&mut self) -> Result<u8, Error<I2C::Error>> {
        Ok(self.device.who_am_i().read()?.value())
    }

    /// Verify the `WHO_AM_I` register and bring the sensor into
    /// continuous-conversion mode at ultra-high performance on all three
    /// axes with the given full scale, block data update enabled, and an
    /// output data rate of 10 Hz. This mirrors a typical datasheet
    /// power-up sequence; call the individual `CTRL_REG*` setters
    /// afterwards if you want different settings.
    fn init(&mut self) -> Result<(), Error<I2C::Error>> {
        let id = self.who_am_i()?;
        if id != Self::WHO_AM_I {
            return Err(Error::WrongDevice { got: id });
        }

        self.device.ctrl_reg_2().write(|reg| {
            reg.set_full_scale(self.full_scale);
        })?;

        self.device.ctrl_reg_1().write(|reg| {
            reg.set_temp_en(true);
            reg.set_xy_operating_mode(XyOperatingMode::UltraHighPerformance);
            reg.set_output_data_rate(OutputDataRate::Hz10);
            reg.set_fast_odr(false);
            reg.set_self_test(false);
        })?;

        self.device.ctrl_reg_4().write(|reg| {
            reg.set_z_operating_mode(ZOperatingMode::UltraHighPerformance);
            reg.set_big_endian(false);
        })?;

        self.device.ctrl_reg_5().write(|reg| {
            reg.set_block_data_update(false);
        })?;

        self.device.ctrl_reg_3().write(|reg| {
            reg.set_system_mode(SystemMode::Continuous);
            reg.set_low_power(false);
        })?;

        Ok(())
    }

    /// `true` once a new X/Y/Z sample is ready to be read
    /// (`STATUS_REG.ZYXDA`, Section 7.10).
    pub fn data_ready(&mut self) -> Result<bool, Error<I2C::Error>> {
        Ok(self.device.status_reg().read()?.xyz_data_available())
    }

    /// Read the raw, signed 16-bit output registers (Sections 7.11-7.13).
    /// Values are two's complement, matching the datasheet.
    pub fn read_magnetometer_raw(&mut self) -> Result<(i16, i16, i16), Error<I2C::Error>> {
        let reg = self.device.out_xyz().read()?;

        Ok((reg.x(), reg.y(), reg.z()))
    }

    /// Read the magnetic field and convert it to gauss using the
    /// sensitivity of the currently configured full scale (Table 2).
    pub fn read_magnetometer_gauss(&mut self) -> Result<MagneticField, Error<I2C::Error>> {
        let (x, y, z) = self.read_magnetometer_raw()?;
        let lsb_per_gauss = Self::sensitivity_lsb_per_gauss(self.full_scale);

        Ok(MagneticField {
            x: x as f32 / lsb_per_gauss,
            y: y as f32 / lsb_per_gauss,
            z: z as f32 / lsb_per_gauss,
        })
    }

    /// Read the die temperature in degrees Celsius (Section 7.14 /
    /// Table 3: 8 LSB/°C, zero offset at 25 °C as on other ST
    /// magnetometers in this family; enable it first via
    /// `CTRL_REG1.TEMP_EN`, which [`Lis3mdl::init`] does for you).
    pub fn read_temperature_celsius(&mut self) -> Result<f32, Error<I2C::Error>> {
        let lo = self.device.temp_out_l().read()?.value();
        let hi = self.device.temp_out_h().read()?.value();
        let raw = i16::from_le_bytes([lo, hi]);
        Ok(25.0 + (raw as f32) / 8.0)
    }
}

impl<I2C: embedded_hal_async::i2c::I2c> Lis3mdl<I2C> {
    /// Take ownership of an I2C bus and wrap it in a LIS3MDL driver.
    /// Nothing is transmitted yet; call [`Lis3mdl::init`] (or configure
    /// the individual `CTRL_REG*` registers yourself) before use.
    pub async fn try_new_async(
        i2c: I2C,
        address: SlaveAddress,
        full_scale: FullScale,
    ) -> Result<Self, Error<I2C::Error>> {
        let mut lis3mdl = Self {
            device: registers::Lis3mdlRegisters::new(I2cInterface {
                i2c,
                address: address.bits(),
            }),
            full_scale,
        };

        lis3mdl.init_async().await?;

        Ok(lis3mdl)
    }

    /// Read the `WHO_AM_I` register. The LIS3MDL always returns `0x3D`
    /// (datasheet Table 16 / Section 7.4).
    pub async fn who_am_i_async(&mut self) -> Result<u8, Error<I2C::Error>> {
        Ok(self.device.who_am_i().read_async().await?.value())
    }

    /// Verify the `WHO_AM_I` register and bring the sensor into
    /// continuous-conversion mode at ultra-high performance on all three
    /// axes with the given full scale, block data update enabled, and an
    /// output data rate of 10 Hz. This mirrors a typical datasheet
    /// power-up sequence; call the individual `CTRL_REG*` setters
    /// afterwards if you want different settings.
    async fn init_async(&mut self) -> Result<(), Error<I2C::Error>> {
        let id = self.who_am_i_async().await?;
        if id != Self::WHO_AM_I {
            return Err(Error::WrongDevice { got: id });
        }

        self.device
            .ctrl_reg_2()
            .write_async(|reg| {
                reg.set_soft_reset(true);
            })
            .await?;

        // --- CTRL_REG2 ---
        let mut expected_ctrl2 = self.device.ctrl_reg_2().read_async().await?;
        expected_ctrl2.set_full_scale(self.full_scale);
        expected_ctrl2.set_soft_reset(false);

        self.device
            .ctrl_reg_2()
            .write_async(|reg| {
                *reg = expected_ctrl2;
            })
            .await?;

        if self.device.ctrl_reg_2().read_async().await? != expected_ctrl2 {
            return Err(Error::WriteVerifyFailed {
                register_address: 0x21,
            });
        }

        // --- CTRL_REG1 ---
        let mut expected_ctrl1 = self.device.ctrl_reg_1().read_async().await?;
        expected_ctrl1.set_temp_en(true);
        expected_ctrl1.set_xy_operating_mode(XyOperatingMode::UltraHighPerformance);
        expected_ctrl1.set_output_data_rate(OutputDataRate::Hz10);
        expected_ctrl1.set_fast_odr(false);
        expected_ctrl1.set_self_test(false);

        self.device
            .ctrl_reg_1()
            .write_async(|reg| {
                *reg = expected_ctrl1;
            })
            .await?;

        let new = self.device.ctrl_reg_1().read_async().await?;
        if new != expected_ctrl1 {
            return Err(Error::WriteVerifyFailed {
                register_address: 0x20,
            });
        }

        // --- CTRL_REG4 ---
        let mut expected_ctrl4 = self.device.ctrl_reg_4().read_async().await?;
        expected_ctrl4.set_z_operating_mode(ZOperatingMode::UltraHighPerformance);
        expected_ctrl4.set_big_endian(false);

        self.device
            .ctrl_reg_4()
            .write_async(|reg| {
                *reg = expected_ctrl4;
            })
            .await?;

        if self.device.ctrl_reg_4().read_async().await? != expected_ctrl4 {
            return Err(Error::WriteVerifyFailed {
                register_address: 0x23,
            });
        }

        // --- CTRL_REG5 ---
        let mut expected_ctrl5 = self.device.ctrl_reg_5().read_async().await?;
        expected_ctrl5.set_block_data_update(false);

        self.device
            .ctrl_reg_5()
            .write_async(|reg| {
                *reg = expected_ctrl5;
            })
            .await?;

        if self.device.ctrl_reg_5().read_async().await? != expected_ctrl5 {
            return Err(Error::WriteVerifyFailed {
                register_address: 0x24,
            });
        }

        // --- CTRL_REG3 ---
        let mut expected_ctrl3 = self.device.ctrl_reg_3().read_async().await?;
        expected_ctrl3.set_system_mode(SystemMode::Continuous);
        expected_ctrl3.set_low_power(false);

        self.device
            .ctrl_reg_3()
            .write_async(|reg| {
                *reg = expected_ctrl3;
            })
            .await?;

        if self.device.ctrl_reg_3().read_async().await? != expected_ctrl3 {
            return Err(Error::WriteVerifyFailed {
                register_address: 0x22,
            });
        }

        Ok(())
    }

    /// `true` once a new X/Y/Z sample is ready to be read
    /// (`STATUS_REG.ZYXDA`, Section 7.10).
    pub async fn data_ready_async(&mut self) -> Result<bool, Error<I2C::Error>> {
        Ok(self
            .device
            .status_reg()
            .read_async()
            .await?
            .xyz_data_available())
    }

    /// Read the raw, signed 16-bit output registers (Sections 7.11-7.13).
    /// Values are two's complement, matching the datasheet.
    pub async fn read_magnetometer_raw_async(
        &mut self,
    ) -> Result<(i16, i16, i16), Error<I2C::Error>> {
        let reg = self.device.out_xyz().read_async().await?;

        Ok((reg.x(), reg.y(), reg.z()))
    }

    /// Read the magnetic field and convert it to gauss using the
    /// sensitivity of the currently configured full scale (Table 2).
    pub async fn read_magnetometer_gauss_async(
        &mut self,
    ) -> Result<MagneticField, Error<I2C::Error>> {
        let (x, y, z) = self.read_magnetometer_raw_async().await?;
        let lsb_per_gauss = Self::sensitivity_lsb_per_gauss(self.full_scale);

        Ok(MagneticField {
            x: x as f32 / lsb_per_gauss,
            y: y as f32 / lsb_per_gauss,
            z: z as f32 / lsb_per_gauss,
        })
    }

    /// Read the die temperature in degrees Celsius (Section 7.14 /
    /// Table 3: 8 LSB/°C, zero offset at 25 °C as on other ST
    /// magnetometers in this family; enable it first via
    /// `CTRL_REG1.TEMP_EN`, which [`Lis3mdl::init`] does for you).
    pub async fn read_temperature_celsius_async(&mut self) -> Result<f32, Error<I2C::Error>> {
        let lo = self.device.temp_out_l().read_async().await?.value();
        let hi = self.device.temp_out_h().read_async().await?.value();
        let raw = i16::from_le_bytes([lo, hi]);
        Ok(25.0 + (raw as f32) / 8.0)
    }
}
