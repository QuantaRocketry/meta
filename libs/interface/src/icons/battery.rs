use embedded_graphics::geometry::OriginDimensions;
use embedded_graphics::image::ImageDrawable;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};

#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub struct BatteryIcon<C> {
    pub color: C,
    pub charge: f32,
}

impl<C> OriginDimensions for BatteryIcon<C> {
    fn size(&self) -> Size {
        (12, 7).into()
    }
}

impl<C: PixelColor> ImageDrawable for BatteryIcon<C> {
    type Color = C;

    fn draw<D>(&self, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        // Top and bottom
        {
            Rectangle::new((1, 0).into(), (9, 1).into())
                .into_styled(PrimitiveStyle::with_fill(self.color))
                .draw(target)?;
            Rectangle::new((1, 6).into(), (9, 1).into())
                .into_styled(PrimitiveStyle::with_fill(self.color))
                .draw(target)?;
        }

        // Left edge
        Rectangle::new((0, 1).into(), (1, 5).into())
            .into_styled(PrimitiveStyle::with_fill(self.color))
            .draw(target)?;

        // Right edge
        {
            Rectangle::new((10, 1).into(), (1, 2).into())
                .into_styled(PrimitiveStyle::with_fill(self.color))
                .draw(target)?;
            Rectangle::new((11, 2).into(), (1, 3).into())
                .into_styled(PrimitiveStyle::with_fill(self.color))
                .draw(target)?;
            Rectangle::new((10, 4).into(), (1, 2).into())
                .into_styled(PrimitiveStyle::with_fill(self.color))
                .draw(target)?;
        }

        // State
        let width = libm::roundf(self.charge * 7.0) as u32;
        Rectangle::new((2, 2).into(), (width, 3).into())
            .into_styled(PrimitiveStyle::with_fill(self.color))
            .draw(target)?;

        Ok(())
    }

    fn draw_sub_image<D>(&self, target: &mut D, area: &Rectangle) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        self.draw(&mut target.translated(-area.top_left).clipped(area))
    }
}
