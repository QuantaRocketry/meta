use core::fmt::Debug;

use embedded_graphics::pixelcolor::BinaryColor;

use crate::{
    event::{InterfaceEvent, NavigateEvent},
    icons::BatteryIcon,
    page::Page,
};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct InterfaceState {
    pub tab: i8,
    pub page: Page,
    pub stop_on_weight: bool,
    pub auto_off: bool,
    pub auto_brew: bool,
    pub clean_overlay: Option<CleanSettings>,
    pub clean_settings: CleanSettings,
    pub location: TrackedPOI,
    pub battery_percentage: f32,
    pub poi: TrackedPOI,
    pub poi_selector: Option<TrackedPOI>,
    pub heading: f32,
    pub battery_widget: BatteryIcon<BinaryColor>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct TrackedPOI {
    pub id: heapless::string::String<16>,
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CleanSettings {
    pub frequency: u32,
    pub time: u32,
}
