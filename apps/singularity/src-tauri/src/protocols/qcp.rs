use std::time::{Duration, Instant};

use serialport::SerialPort;

use super::{Context, ProtocolHandler};

const UPTIME_POLL_INTERVAL: Duration = Duration::from_secs(1);

/// COBS-framed postcard messages. Polls the device for uptime once a second.
#[derive(Default)]
pub struct Qcp {
    rx_buf: Vec<u8>,
    last_request: Option<Instant>,
}

impl ProtocolHandler for Qcp {
    fn poll(&mut self, port: &mut dyn SerialPort) -> Result<(), String> {
        let due = self
            .last_request
            .map_or(true, |t| t.elapsed() >= UPTIME_POLL_INTERVAL);
        if due {
            self.last_request = Some(Instant::now());
            send_request(port, ::qcp::MessageKind::Uptime)?;
        }
        Ok(())
    }

    fn on_data(&mut self, bytes: &[u8], ctx: &mut Context) {
        self.rx_buf.extend_from_slice(bytes);
        for message in drain_frames(&mut self.rx_buf) {
            ctx.push_line(format_message(message));
        }
    }
}

fn format_message(message: ::qcp::Message) -> String {
    match message {
        ::qcp::Message::Uptime(us) => format_uptime(us),
        other => format!("{:?}", other),
    }
}

fn format_uptime(micros: u64) -> String {
    let total_ms = micros / 1000;
    let (ms, secs) = (total_ms % 1000, total_ms / 1000);
    format!(
        "Uptime: {:02}:{:02}:{:02}.{:03}",
        secs / 3600,
        (secs / 60) % 60,
        secs % 60,
        ms
    )
}

/// Send a COBS-framed request over `port`.
fn send_request(port: &mut dyn SerialPort, kind: ::qcp::MessageKind) -> Result<(), String> {
    let mut buf = [0u8; ::qcp::MESSAGE_SIZE_MAX + 4];
    let frame = postcard::to_slice_cobs(&::qcp::Message::Request(kind), &mut buf)
        .map_err(|e| e.to_string())?;
    port.write_all(frame).map_err(|e| e.to_string())?;
    port.flush().map_err(|e| e.to_string())
}

/// Pull complete zero-delimited frames out of `frames`, decoding each into a
/// message. Malformed frames are dropped.
fn drain_frames(frames: &mut Vec<u8>) -> Vec<::qcp::Message> {
    let mut messages = Vec::new();
    while let Some(end) = frames.iter().position(|&b| b == 0) {
        let mut frame: Vec<u8> = frames.drain(..=end).collect();
        frame.pop();
        if frame.is_empty() {
            continue;
        }
        if let Ok(message) = postcard::from_bytes_cobs::<::qcp::Message>(&mut frame) {
            messages.push(message);
        }
    }
    messages
}
