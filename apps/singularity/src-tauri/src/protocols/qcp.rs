use std::time::{Duration, Instant};

use serde::Serialize;
use serialport::SerialPort;
use tauri::Emitter;

use super::{Context, ProtocolHandler};

/// The device's answer to a radio config, for the Configuration page.
#[derive(Clone, Serialize)]
struct RadioConfigResult {
    ok: bool,
    message: String,
}

const UPTIME_POLL_INTERVAL: Duration = Duration::from_secs(1);

/// COBS-framed postcard messages. Polls the device for uptime once a second.
#[derive(Default)]
pub struct Qcp {
    rx_buf: Vec<u8>,
    /// Received radio payload text not yet ended by a newline. Eggtimer
    /// sends an NMEA stream in chunks, so sentences span several payloads.
    radio_text: Vec<u8>,
    last_request: Option<Instant>,
}

impl ProtocolHandler for Qcp {
    fn poll(&mut self, port: &mut dyn SerialPort) -> Result<(), String> {
        let due = self
            .last_request
            .map_or(true, |t| t.elapsed() >= UPTIME_POLL_INTERVAL);
        if due {
            self.last_request = Some(Instant::now());
            send_message(port, &::qcp::Message::Request(::qcp::MessageKind::Uptime))?;
        }
        Ok(())
    }

    fn on_data(&mut self, bytes: &[u8], ctx: &mut Context) {
        self.rx_buf.extend_from_slice(bytes);
        for message in drain_frames(&mut self.rx_buf) {
            match message {
                ::qcp::Message::RawPayload(payload) => {
                    self.radio_text.extend_from_slice(&payload);
                    for line in drain_lines(&mut self.radio_text) {
                        ctx.push_line(line);
                    }
                }
                other => {
                    let line = format_message(other.clone());
                    match other {
                        ::qcp::Message::Protocol(_) | ::qcp::Message::RadioConfig(_) => {
                            let _ = ctx.app.emit(
                                "radio_config_result",
                                RadioConfigResult { ok: true, message: line.clone() },
                            );
                        }
                        ::qcp::Message::Error(_) => {
                            let _ = ctx.app.emit(
                                "radio_config_result",
                                RadioConfigResult { ok: false, message: line.clone() },
                            );
                        }
                        _ => {}
                    }
                    ctx.push_line(line);
                }
            }
        }
    }
}

fn format_message(message: ::qcp::Message) -> String {
    match message {
        ::qcp::Message::Uptime(us) => format_uptime(us),
        ::qcp::Message::Protocol(::qcp::Protocol::Eggtimer(p)) => format!(
            "Radio applied: eggtimer, centre {:.3} MHz, channel {} ({:.3} MHz)",
            p.center_frequency_hz as f64 / 1e6,
            p.channel.number(),
            p.carrier_frequency_hz() as f64 / 1e6,
        ),
        ::qcp::Message::RadioConfig(config) => format!("Radio applied: {:?}", config),
        ::qcp::Message::Error(e) => format!("Device rejected the radio config: {:?}", e),
        other => format!("{:?}", other),
    }
}

/// Pull complete lines out of `text`, dropping the line endings.
fn drain_lines(text: &mut Vec<u8>) -> Vec<String> {
    let mut lines = Vec::new();
    while let Some(end) = text.iter().position(|&b| b == b'\n') {
        let line: Vec<u8> = text.drain(..=end).collect();
        lines.push(String::from_utf8_lossy(&line).trim_end().to_string());
    }
    lines
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

/// Send `message` over `port` as one COBS frame.
pub fn send_message(port: &mut dyn SerialPort, message: &::qcp::Message) -> Result<(), String> {
    let mut buf = [0u8; ::qcp::FRAME_SIZE_MAX];
    let frame = ::qcp::encode_frame(message, &mut buf).map_err(|e| e.to_string())?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sentences_split_across_payloads_become_one_line() {
        let mut text = b"$GPGGA,1234".to_vec();
        assert!(drain_lines(&mut text).is_empty());
        text.extend_from_slice(b"56,N*4F\r\n$GPR");
        assert_eq!(drain_lines(&mut text), ["$GPGGA,123456,N*4F"]);
        assert_eq!(text, b"$GPR");
    }
}
