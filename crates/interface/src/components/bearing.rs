use embedded_graphics::geometry::OriginDimensions;
use embedded_graphics::image::ImageDrawable;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Circle, PrimitiveStyle, Rectangle, Triangle};

pub struct Bearing<C> {
    /// Color of the drawing
    pub color: C,
    /// Angle of tracked object in degrees relative to the current heading
    pub angle: f32,
}

impl<C> Bearing<C> {
    const SIZE: u32 = 51;
}

impl<C> OriginDimensions for Bearing<C> {
    fn size(&self) -> Size {
        (Self::SIZE, Self::SIZE).into()
    }
}

impl<C: PixelColor> ImageDrawable for Bearing<C> {
    type Color = C;

    fn draw<D>(&self, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        // Dial
        Circle::new((0, 0).into(), Self::SIZE)
            .into_styled(PrimitiveStyle::with_stroke(self.color, 1))
            .draw(target)?;

        // Cursor
        {
            let half = (Self::SIZE / 2) as i32;
            let eighth = (Self::SIZE / 8) as i32;
            let center = (half, half);

            // 0 deg = up, increasing clockwise. In screen space (y-down),
            // the standard rotation matrix applied as-is already produces
            // a clockwise visual rotation, so the angle isn't negated.
            let angle_rad = self.angle.to_radians();
            let (sin_a, cos_a) = (libm::sinf(angle_rad), libm::cosf(angle_rad));

            let rotate = |x: i32, y: i32| -> Point {
                let dx = (x - center.0) as f32;
                let dy = (y - center.1) as f32;
                let rx = dx * cos_a - dy * sin_a;
                let ry = dx * sin_a + dy * cos_a;
                Point::new(
                    center.0 + libm::roundf(rx) as i32,
                    center.1 + libm::roundf(ry) as i32,
                )
            };

            // head (pointed) half
            Triangle::new(
                rotate(half - eighth, half),
                rotate(half + eighth, half),
                rotate(half, half - (eighth * 3)),
            )
            .into_styled(PrimitiveStyle::with_fill(self.color))
            .draw(target)?;

            // tail half
            Triangle::new(
                rotate(half - eighth, half),
                rotate(half + eighth, half),
                rotate(half, half + eighth),
            )
            .into_styled(PrimitiveStyle::with_fill(self.color))
            .draw(target)?;
        }

        Ok(())
    }

    fn draw_sub_image<D>(&self, target: &mut D, area: &Rectangle) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        self.draw(&mut target.translated(-area.top_left).clipped(area))
    }
}
