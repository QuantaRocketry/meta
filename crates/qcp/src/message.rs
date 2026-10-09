use serde::{Deserialize, Serialize};

use crate::config::{radio::Protocol, radio::RadioConfig};
use crate::state::{BatteryState, TrackedPOI};

/// The set of messages that can be sent over the wire.
///
/// More variants can be added here as the protocol grows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Message {
    /// Ask the other end to (re)send a specific kind of message.
    Request(MessageKind),
    /// The uptime of the device in microseconds
    Uptime(u64),
    TrackedPOI(TrackedPOI),
    RadioConfig(RadioConfig),
    Protocol(Protocol),
    /// Battery percentage and charging status. Can be requested with
    /// `Request`.
    Battery(BatteryState),
}

impl Message {
    /// The kind of this message, i.e. what a `Request` would ask for to get
    /// one like it. `None` for `Request` itself, which can't be requested.
    pub fn kind(&self) -> Option<MessageKind> {
        match self {
            Message::Request(_) => None,
            Message::Uptime(_) => Some(MessageKind::Uptime),
            Message::TrackedPOI(_) => Some(MessageKind::TrackedPOI),
            Message::RadioConfig(_) => Some(MessageKind::RadioConfig),
            Message::Protocol(_) => Some(MessageKind::Protocol),
            Message::Battery(_) => Some(MessageKind::Battery),
        }
    }
}

/// The messages that can be asked for with `Message::Request`.
///
/// Sent on the wire as the variant's position in this enum, so only append
/// new variants; reordering or removing breaks compatibility.
/// `Message::kind` is an exhaustive match, so adding a `Message` variant
/// forces a decision about whether it's requestable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageKind {
    Uptime,
    TrackedPOI,
    RadioConfig,
    Protocol,
    Battery,
}

impl From<BatteryState> for Message {
    fn from(state: BatteryState) -> Self {
        Message::Battery(state)
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_round_trip;

    #[test]
    fn request_round_trip() {
        assert_round_trip(&Message::Request(MessageKind::Battery));
    }

    #[test]
    fn request_is_one_byte_per_field() {
        let mut buf = [0u8; 8];
        let encoded = crate::encode(&Message::Request(MessageKind::Battery), &mut buf).unwrap();
        assert_eq!(encoded.len(), 2);
    }
}
