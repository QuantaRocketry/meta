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
    /// Bytes received over the radio, passed on verbatim. Pushed by the
    /// device as packets arrive; can't be requested.
    RawPayload(RawPayload),
    /// The device could not apply a config it was sent. Sent in place of the
    /// echo that acknowledges a config; can't be requested.
    Error(ConfigError),
}

/// Why a device rejected a radio config.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigError {
    /// The radio does not do this modulation (e.g. LoRa on an FSK-only app).
    UnsupportedModulation,
    /// The app does not handle this protocol (e.g. `Quanta`, `Uts`).
    UnsupportedProtocol,
    FrequencyOutOfRange,
    BitRateOutOfRange,
    DeviationOutOfRange,
    /// Only variable-length packets are supported.
    FixedLength,
    /// Manchester and whitening are not supported.
    DataCoding,
    /// The config is valid but the radio failed to take it.
    RadioFault,
}

/// The largest radio packet payload a `RawPayload` can carry.
pub const RAW_PAYLOAD_MAX: usize = 64;

/// The payload of one received radio packet.
pub type RawPayload = heapless::Vec<u8, RAW_PAYLOAD_MAX>;

impl Message {
    /// The kind of this message, i.e. what a `Request` would ask for to get
    /// one like it. `None` for `Request` itself and for `RawPayload` and `Error`,
    /// which can't be requested.
    pub fn kind(&self) -> Option<MessageKind> {
        match self {
            Message::Request(_) => None,
            Message::Uptime(_) => Some(MessageKind::Uptime),
            Message::TrackedPOI(_) => Some(MessageKind::TrackedPOI),
            Message::RadioConfig(_) => Some(MessageKind::RadioConfig),
            Message::Protocol(_) => Some(MessageKind::Protocol),
            Message::Battery(_) => Some(MessageKind::Battery),
            Message::RawPayload(_) | Message::Error(_) => None,
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
    use crate::{MESSAGE_SIZE_MAX, assert_round_trip, encode};

    #[test]
    fn error_round_trip() {
        assert_round_trip(&Message::Error(ConfigError::DataCoding));
    }

    #[test]
    fn full_raw_payload_fits_in_a_message() {
        let payload = RawPayload::from_slice(&[0xFF; RAW_PAYLOAD_MAX]).unwrap();
        let message = Message::RawPayload(payload);
        let mut buf = [0u8; MESSAGE_SIZE_MAX];
        assert!(encode(&message, &mut buf).is_ok());
        assert_round_trip(&message);
        assert_eq!(message.kind(), None);
    }

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
