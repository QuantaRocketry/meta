use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii},
    prelude::*,
    primitives::{Circle, Line, PrimitiveStyle},
    text::Text,
};

pub struct DisplayTheme<C> {
    pub primary: C,
    pub secondary: C,
    pub background: C,
}

impl<C: RgbColor> Default for DisplayTheme<C> {
    fn default() -> Self {
        Self {
            primary: RgbColor::BLACK,
            secondary: RgbColor::BLACK,
            background: RgbColor::WHITE,
        }
    }
}

// Signal of a point of interest
#[derive(Clone)]
pub struct Signal<'a> {
    pub uid: &'a str,
    pub snr: i32,
    pub rssi: i32,
    pub angle: f32,
    pub distance: f32,
}

pub struct Display<'a, D, C> {
    /// The target to draw to
    target: D,
    /// The theme to apply to the display
    theme: DisplayTheme<C>,
    /// The margin from the edge of the target in px
    margin: i32,
    /// The current POI being tracked
    selected: Option<&'a str>,
}

impl<'a, D, C> Display<'a, D, C>
where
    D: DrawTarget<Color = C>,
    C: PixelColor,
{
    pub fn new(target: D, theme: DisplayTheme<C>, margin: i32) -> Self {
        Display {
            target,
            theme,
            margin,
            selected: None,
        }
    }

    /// Sets the UID of the device being tracked
    pub fn select_uid(&mut self, uid: &'a str) {
        self.selected = Some(uid);
    }

    /// Sets the UID of the device being tracked
    pub fn select_signal(&mut self, signal: &Signal<'a>) {
        self.selected = Some(signal.uid);
    }

    pub fn get_target(&self) -> &D {
        &self.target
    }

    pub fn draw(&mut self, signals: &[Signal<'a>]) -> Result<(), D::Error> {
        let _ = self.target.clear(self.theme.background);

        let bounding_box = self.target.bounding_box();
        let diameter =
            bounding_box.size.width.min(bounding_box.size.height) - 2 * self.margin as u32;
        let radar = Circle::with_center(bounding_box.center(), diameter);

        let min_dist = signals
            .iter()
            .map(|s| s.distance)
            .fold(f32::INFINITY, |a, b| a.min(b));
        let max_dist = signals
            .iter()
            .map(|s| s.distance)
            .fold(0f32, |a, b| a.max(b))
            * 1.1; // Add some scale to force inside border

        // NOTE: In no-std environments, consider using
        // [arrayvec](https://stackoverflow.com/a/39491059/383609) and a fixed size buffer

        if let Some(signal) = self.selected {
            for s in signals {
                if s.uid == signal {
                    self.draw_uid(signal)?;
                    self.draw_snr(Some(s.snr))?;
                    self.draw_rssi(Some(s.rssi))?;
                    self.draw_signal(radar, max_dist, s, true)?;
                } else {
                    self.draw_signal(radar, max_dist, s, false)?;
                }
            }
        } else {
            self.draw_uid("")?;
            self.draw_snr(None)?;
            self.draw_rssi(None)?;
            for s in signals {
                self.draw_signal(radar, max_dist, s, false)?;
            }
        }

        radar
            .into_styled(PrimitiveStyle::with_stroke(self.theme.secondary, 1))
            .draw(&mut self.target)?;

        Circle::with_center(radar.center(), 6)
            .into_styled(PrimitiveStyle::with_fill(self.theme.primary))
            .draw(&mut self.target)?;

        Ok(())
    }

    fn draw_uid(&mut self, uid: &str) -> Result<(), D::Error> {
        // Create a styled text object for the id.
        let mut text = Text::new(
            uid,
            Point::zero(),
            MonoTextStyle::new(&ascii::FONT_7X13, self.theme.primary),
        );

        // Move text to be within margin
        text.translate_mut(
            text.position
                + text.bounding_box().size.y_axis() / 2
                + Point::new(self.margin, self.margin),
        );
        text.draw(&mut self.target)?;

        Ok(())
    }

    fn draw_snr(&mut self, snr: Option<i32>) -> Result<(), D::Error> {
        let tag = if let Some(s) = snr {
            format!("SNR:{}dB", s)
        } else {
            format!("SNR:-dB")
        };

        // Create a styled text object for the id.
        let mut text = Text::new(
            &tag,
            Point::zero(),
            MonoTextStyle::new(&ascii::FONT_6X13, self.theme.primary),
        );

        // Move text to be within margin
        text.translate_mut(
            self.target
                .bounding_box()
                .bottom_right()
                .ok_or("should have size")
                .unwrap()
                - text.bounding_box().size.x_axis()
                - Point::new(self.margin, self.margin),
        );
        text.draw(&mut self.target)?;

        Ok(())
    }

    fn draw_rssi(&mut self, rssi: Option<i32>) -> Result<(), D::Error> {
        let tag = if let Some(val) = rssi {
            format!("RSSI:{}dBm", val)
        } else {
            format!("RSSI:-dBm")
        };

        let mut text = Text::new(
            &tag,
            Point::zero(),
            MonoTextStyle::new(&ascii::FONT_6X13, self.theme.primary),
        );

        text.translate_mut(
            self.target.bounding_box().top_left
                + self.target.bounding_box().size.y_axis()
                // + text.bounding_box().size.y_axis() / 2
                + Point::new(self.margin, -self.margin),
        );
        text.draw(&mut self.target)?;

        Ok(())
    }

    fn draw_distance(&mut self, distance: u32) -> Result<(), D::Error> {
        let tag = format!("{}m", distance);
        let mut text = Text::new(
            &tag,
            Point::zero(),
            MonoTextStyle::new(&ascii::FONT_6X13, self.theme.primary),
        );

        text.translate_mut(
            self.target.bounding_box().top_left
                + self.target.bounding_box().size.y_axis()
                // + text.bounding_box().size.y_axis() / 2
                + Point::new(self.margin, -self.margin),
        );
        text.draw(&mut self.target)?;

        Ok(())
    }

    fn draw_signal(
        &mut self,
        radar: Circle,
        radar_radius: f32,
        signal: &Signal<'a>,
        is_selected: bool,
    ) -> Result<(), D::Error> {
        let length = signal.distance / radar_radius * (radar.diameter as f32 / 2.0);

        let style = if is_selected {
            PrimitiveStyle::with_stroke(self.theme.primary, 1)
        } else {
            PrimitiveStyle::with_stroke(self.theme.secondary, 1)
        };

        let line = bearing_to_line(radar.center(), signal.angle, length);
        line.into_styled(style).draw(&mut self.target)
    }
}

/// Converts a polar coordinate (angle/distance) into an (X, Y) coordinate centered around the
/// center of the circle.
///
/// The angle is relative to the 12 o'clock position and the radius is relative to the edge of the
/// clock face.
fn polar(circle: &Circle, angle: f32, radius: f32) -> Point {
    circle.center()
        + Point::new(
            (angle.sin() * radius) as i32,
            -(angle.cos() * radius) as i32,
        )
}

fn bearing_to_line(from: Point, angle: f32, distance: f32) -> Line {
    Line::new(
        from,
        from + Point::new(
            (angle.to_radians().sin() * distance) as i32,
            -(angle.to_radians().cos() * distance) as i32,
        ),
    )
}

/// Converts an hour into an angle in radians.
// fn hour_to_angle(hour: u32) -> f32 {
//     // Convert from 24 to 12 hour time.
//     let hour = hour % 12;

//     (hour as f32 / 12.0) * 2.0 * PI
// }

/// Converts a sexagesimal (base 60) value into an angle in radians.
// fn sexagesimal_to_angle(value: u32) -> f32 {
//     (value as f32 / 60.0) * 2.0 * PI
// }

#[cfg(test)]
mod tests {
    use super::*;
}
