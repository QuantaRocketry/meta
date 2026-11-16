use serde::{Deserialize, Serialize};

use crate::coordinate::Coordinate;

/// A point of interest being tracked, identified by a short callsign/ID and
/// its last known position.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackedPOI {
    pub id: heapless::String<16>,
    pub coordinate: Coordinate,
}

/// Whether the battery is currently being charged.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChargeStatus {
    #[default]
    Discharging,
    Charging,
    Full,
}

/// Battery charge level and charging status.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatteryState {
    /// Charge level, 0 (empty) to 100 (full).
    pub percent: u8,
    pub status: ChargeStatus,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MESSAGE_SIZE_MAX, Message, assert_round_trip, encode};

    #[test]
    fn battery_round_trip() {
        assert_round_trip(
            &BatteryState {
                percent: 87,
                status: ChargeStatus::Charging,
            }
            .into(),
        );
    }

    fn sample_poi() -> TrackedPOI {
        let mut poi = TrackedPOI {
            coordinate: Coordinate {
                latitude: 33.123123,
                longitude: -151.456456,
                altitude: 12.5,
            },
            ..Default::default()
        };
        poi.id.push_str("VK2GTX").unwrap();
        poi
    }

    #[test]
    fn round_trip() {
        assert_round_trip(&sample_poi().into());
    }

    #[test]
    fn fits_within_message_size_max() {
        let mut poi = sample_poi();
        // Worst case: a fully-populated, maximum-length id.
        poi.id.clear();
        poi.id.push_str("0123456789ABCDEF").unwrap();
        let message: Message = poi.into();

        let mut buf = [0u8; MESSAGE_SIZE_MAX];
        let encoded = encode(&message, &mut buf).unwrap();
        assert!(encoded.len() <= MESSAGE_SIZE_MAX);
    }
}
