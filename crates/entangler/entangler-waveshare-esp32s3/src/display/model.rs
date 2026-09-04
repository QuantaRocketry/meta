use mipidsi::{
    dcs::{self, InterfaceExt},
    models::Model,
};

pub struct JD9853;

impl Model for JD9853 {
    type ColorFormat = embedded_graphics::pixelcolor::Rgb565;

    const FRAMEBUFFER_SIZE: (u16, u16) = (240, 320);

    fn init<DELAY, DI>(
        &mut self,
        di: &mut DI,
        delay: &mut DELAY,
        options: &mipidsi::options::ModelOptions,
    ) -> Result<mipidsi::dcs::SetAddressMode, DI::Error>
    where
        DELAY: embedded_hal::delay::DelayNs,
        DI: mipidsi::interface::Interface,
    {
        let madctl = dcs::SetAddressMode::from(options);

        delay.delay_us(150_000);

        di.write_command(dcs::ExitSleepMode)?;
        delay.delay_us(10_000);

        // set hw scroll area based on framebuffer size
        di.write_command(madctl)?;

        di.write_command(dcs::SetInvertMode::new(options.invert_colors))?;

        let pf =
            dcs::PixelFormat::with_all(dcs::BitsPerPixel::from_rgb_color::<Self::ColorFormat>());
        di.write_command(dcs::SetPixelFormat::new(pf))?;
        delay.delay_us(10_000);
        di.write_command(dcs::EnterNormalMode)?;
        delay.delay_us(10_000);
        di.write_command(dcs::SetDisplayOn)?;

        // DISPON requires some time otherwise we risk SPI data issues
        delay.delay_us(120_000);

        Ok(madctl)
    }
}
