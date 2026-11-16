use serde::{Deserialize, Serialize};

/// A position on the WGS84 ellipsoid.
#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Coordinate {
    /// Latitude in degrees, positive north.
    pub latitude: f64,
    /// Longitude in degrees, positive east.
    pub longitude: f64,
    /// Altitude in meters above the WGS84 ellipsoid.
    pub altitude: f32,
}
