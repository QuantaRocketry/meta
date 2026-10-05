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
