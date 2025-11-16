use chrono::{Local, Timelike};
use core::f32::consts::PI;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_9X15},
    prelude::*,
    primitives::{Circle, Line, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle},
    text::Text,
};

pub struct DisplayTheme<C> {
    pub primary: C,
    pub background: C,
}

impl<C: RgbColor> Default for DisplayTheme<C> {
    fn default() -> Self {
        Self {
            primary: RgbColor::WHITE,
            background: RgbColor::BLACK,
        }
    }
}

pub struct Display<D, C> {
    target: D,
    theme: DisplayTheme<C>,
    clock_face: Circle,
    margin: u32,
}

impl<D, C> Display<D, C>
where
    D: DrawTarget<Color = C>,
    C: PixelColor,
{
    pub fn new(target: D, theme: DisplayTheme<C>, margin: u32) -> Self {
        let bounding_box = target.bounding_box();
        let diameter = bounding_box.size.width.min(bounding_box.size.height) - 2 * margin;
        let clock_face = Circle::with_center(bounding_box.center(), diameter);

        Display {
            target,
            theme,
            clock_face,
            margin,
        }
    }

    pub fn get_target(&self) -> &D {
        &self.target
    }

    pub fn draw(&mut self, time: chrono::DateTime<Local>) -> Result<(), D::Error> {
        let _ = self.target.clear(self.theme.background);

        // Calculate the position of the three clock hands in radians.
        let hours_radians = hour_to_angle(time.hour());
        let minutes_radians = sexagesimal_to_angle(time.minute());
        let seconds_radians = sexagesimal_to_angle(time.second());

        // NOTE: In no-std environments, consider using
        // [arrayvec](https://stackoverflow.com/a/39491059/383609) and a fixed size buffer
        let digital_clock_text = format!(
            "{:02}:{:02}:{:02}",
            time.hour(),
            time.minute(),
            time.second()
        );

        self.draw_face()?;
        self.draw_hand(hours_radians, -60)?;
        self.draw_hand(minutes_radians, -30)?;
        self.draw_hand(seconds_radians, 0)?;
        self.draw_second_decoration(seconds_radians, -20)?;

        self.draw_digital_clock(&digital_clock_text)?;

        // Draw a small circle over the hands in the center of the clock face.
        // This has to happen after the hands are drawn so they're covered up.
        Circle::with_center(self.clock_face.center(), 9)
            .into_styled(PrimitiveStyle::with_fill(self.theme.primary))
            .draw(&mut self.target)?;

        Ok(())
    }

    pub fn draw_hand(&mut self, angle: f32, length_delta: i32) -> Result<(), D::Error> {
        let end = polar(&self.clock_face, angle, length_delta);

        Line::new(self.clock_face.center(), end)
            .into_styled(PrimitiveStyle::with_stroke(self.theme.primary, 1))
            .draw(&mut self.target)
    }

    /// Draws a circle and 12 graduations as a simple clock face.
    fn draw_face(&mut self) -> Result<(), D::Error> {
        // Draw the outer face.
        self.clock_face
            .into_styled(PrimitiveStyle::with_stroke(self.theme.primary, 2))
            .draw(&mut self.target)?;

        // Draw 12 graduations.
        for angle in (0..12).map(hour_to_angle) {
            // Start point on circumference.
            let start = polar(&self.clock_face, angle, 0);

            // End point offset by 10 pixels from the edge.
            let end = polar(&self.clock_face, angle, -10);

            Line::new(start, end)
                .into_styled(PrimitiveStyle::with_stroke(self.theme.primary, 1))
                .draw(&mut self.target)?;
        }

        Ok(())
    }

    /// Draws a decorative circle on the second hand.
    fn draw_second_decoration(&mut self, angle: f32, length_delta: i32) -> Result<(), D::Error> {
        let decoration_position = polar(&self.clock_face, angle, length_delta);

        let decoration_style = PrimitiveStyleBuilder::new()
            .fill_color(self.theme.background)
            .stroke_color(self.theme.primary)
            .stroke_width(1)
            .build();

        // Draw a fancy circle near the end of the second hand.
        Circle::with_center(decoration_position, 11)
            .into_styled(decoration_style)
            .draw(&mut self.target)
    }

    /// Draw digital clock just above center with black text on a white background
    fn draw_digital_clock(&mut self, time_str: &str) -> Result<(), D::Error> {
        // Create a styled text object for the time text.
        let mut text = Text::new(
            &time_str,
            Point::zero(),
            MonoTextStyle::new(&FONT_9X15, self.theme.background),
        );

        // Move text to be centered between the 12 o'clock point and the center of the clock face.
        text.translate_mut(
            self.clock_face.center()
                - text.bounding_box().center()
                - self.clock_face.bounding_box().size.y_axis() / 4,
        );

        // Add a background around the time digits.
        // Note that there is no bottom-right padding as this is added by the font renderer itself.
        let text_dimensions = text.bounding_box();
        Rectangle::new(
            text_dimensions.top_left - Point::new(3, 3),
            text_dimensions.size + Size::new(4, 4),
        )
        .into_styled(PrimitiveStyle::with_fill(self.theme.primary))
        .draw(&mut self.target)?;

        // Draw the text after the background is drawn.
        text.draw(&mut self.target)?;

        Ok(())
    }
}

/// Converts a polar coordinate (angle/distance) into an (X, Y) coordinate centered around the
/// center of the circle.
///
/// The angle is relative to the 12 o'clock position and the radius is relative to the edge of the
/// clock face.
fn polar(circle: &Circle, angle: f32, radius_delta: i32) -> Point {
    let radius = circle.diameter as f32 / 2.0 + radius_delta as f32;

    circle.center()
        + Point::new(
            (angle.sin() * radius) as i32,
            -(angle.cos() * radius) as i32,
        )
}

/// Converts an hour into an angle in radians.
fn hour_to_angle(hour: u32) -> f32 {
    // Convert from 24 to 12 hour time.
    let hour = hour % 12;

    (hour as f32 / 12.0) * 2.0 * PI
}

/// Converts a sexagesimal (base 60) value into an angle in radians.
fn sexagesimal_to_angle(value: u32) -> f32 {
    (value as f32 / 60.0) * 2.0 * PI
}

#[cfg(test)]
mod tests {
    use super::*;
}
