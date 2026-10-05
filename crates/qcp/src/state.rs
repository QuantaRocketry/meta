use serde::{Deserialize, Serialize};

use crate::coordinate::Coordinate;

/// A point of interest being tracked, identified by a short callsign/ID and
/// its last known position.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackedPOI {
    pub id: heapless::String<16>,
    pub coordinate: Coordinate,
}
