pub mod battery;
pub mod blink;
pub mod gnss;
pub mod radio;
pub mod usb;

#[cfg(feature = "has-buzzer")]
pub mod buzzer;

#[cfg(feature = "has-oled")]
pub mod oled;

#[cfg(feature = "has-joystick")]
pub mod joystick;

#[cfg(feature = "has-compass")]
pub mod compass;
