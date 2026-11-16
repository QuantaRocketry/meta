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

impl Protocol {
    /// The waveform a receiver tunes to for this protocol, or `None` for a
    /// protocol whose waveform is not defined yet.
    pub fn radio_config(self) -> Option<crate::RadioConfig> {
        match self {
            Protocol::Eggtimer(eggtimer) => Some(eggtimer.into()),
            Protocol::Quanta | Protocol::Uts => None,
        }
    }
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
