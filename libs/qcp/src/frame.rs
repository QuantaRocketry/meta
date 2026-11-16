//! COBS framing of messages on a byte stream: each frame ends in a zero byte.

use crate::{MESSAGE_SIZE_MAX, Message};

/// Room for the largest message plus COBS overhead and the delimiter.
pub const FRAME_SIZE_MAX: usize = MESSAGE_SIZE_MAX + 4;

/// Encode `message` as one frame into `buf`, returning the bytes to send.
pub fn encode_frame<'a>(message: &Message, buf: &'a mut [u8; FRAME_SIZE_MAX]) -> crate::Result<&'a mut [u8]> {
    postcard::to_slice_cobs(message, buf)
}

/// Reassembles frames from a byte stream and decodes them into messages.
/// Oversized, empty and undecodable frames are dropped.
pub struct FrameDecoder {
    buf: [u8; FRAME_SIZE_MAX],
    len: usize,
    overflowed: bool,
}

impl FrameDecoder {
    pub const fn new() -> Self {
        Self { buf: [0; FRAME_SIZE_MAX], len: 0, overflowed: false }
    }

    /// Feed one byte, returning a message once a complete frame decodes.
    pub fn push(&mut self, byte: u8) -> Option<Message> {
        if byte != 0 {
            if self.len < self.buf.len() {
                self.buf[self.len] = byte;
                self.len += 1;
            } else {
                self.overflowed = true;
            }
            return None;
        }

        let len = core::mem::take(&mut self.len);
        if core::mem::take(&mut self.overflowed) || len == 0 {
            return None;
        }
        postcard::from_bytes_cobs(&mut self.buf[..len]).ok()
    }
}

impl Default for FrameDecoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MessageKind;

    fn decode_all(decoder: &mut FrameDecoder, bytes: &[u8]) -> Vec<Message> {
        bytes.iter().filter_map(|&b| decoder.push(b)).collect()
    }

    #[test]
    fn frames_round_trip_across_arbitrary_chunking() {
        let messages = [Message::Request(MessageKind::Protocol), Message::Uptime(1 << 40)];
        let mut wire = Vec::new();
        for m in &messages {
            let mut buf = [0; FRAME_SIZE_MAX];
            wire.extend_from_slice(encode_frame(m, &mut buf).unwrap());
        }
        let mut decoder = FrameDecoder::new();
        assert_eq!(decode_all(&mut decoder, &wire), messages);
    }

    #[test]
    fn garbage_is_dropped_and_the_next_frame_still_decodes() {
        let mut buf = [0; FRAME_SIZE_MAX];
        let good = Message::Uptime(7);
        let mut wire = vec![0, 0xFF, 0x13, 0x37, 0];
        wire.extend(vec![1u8; FRAME_SIZE_MAX * 2]);
        wire.push(0);
        wire.extend_from_slice(encode_frame(&good, &mut buf).unwrap());
        assert_eq!(decode_all(&mut FrameDecoder::new(), &wire), [good]);
    }
}
