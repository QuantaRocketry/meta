use serde::{Deserialize, Serialize};

pub mod eggtimer;
pub mod quanta;
pub mod uts;

/// The application-layer telemetry protocol carried over the link.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Protocol {
    #[default]
    Quanta,
    Eggtimer(eggtimer::Protocol),
    Uts,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_round_trip;

    #[test]
    fn protocol_round_trip() {
        assert_round_trip(&Protocol::Eggtimer(eggtimer::Protocol::default()).into());
    }
}
