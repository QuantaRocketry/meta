use serde::{Deserialize, Serialize};

use crate::config::{radio::Protocol, radio::RadioConfig};
use crate::state::TrackedPOI;

/// The set of messages that can be sent over the wire.
///
/// More variants can be added here as the protocol grows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Message {
    /// Ask the other end to (re)send a specific message variant. The id
    /// identifies which variant is being requested (e.g. by its position in
    /// this enum); it's just an opaque tag for now.
    Request(u8),
    /// The uptime of the device in microseconds
    Uptime(u64),
    TrackedPOI(TrackedPOI),
    RadioConfig(RadioConfig),
    Protocol(Protocol),
}

impl From<TrackedPOI> for Message {
    fn from(poi: TrackedPOI) -> Self {
        Message::TrackedPOI(poi)
    }
}

impl From<RadioConfig> for Message {
    fn from(config: RadioConfig) -> Self {
        Message::RadioConfig(config)
    }
}

impl From<Protocol> for Message {
    fn from(protocol: Protocol) -> Self {
        Message::Protocol(protocol)
    }
}
