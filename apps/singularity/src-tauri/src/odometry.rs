use serde::Serialize;

#[derive(Default, Serialize, Clone)]
pub struct Coordinate {
    pub lat: f32,
    pub lng: f32,
    pub alt: f32,
}

#[derive(Default, Serialize, Clone)]
pub struct OdometryData {
    pub base_station: Coordinate,
    pub poi: Coordinate,
}
