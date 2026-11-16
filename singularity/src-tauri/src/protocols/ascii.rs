use super::{Context, ProtocolHandler};

/// Line-based text. Every newline-separated chunk of a read becomes a line.
pub struct Ascii;

impl ProtocolHandler for Ascii {
    fn on_data(&mut self, bytes: &[u8], ctx: &mut Context) {
        for line in bytes.split(|&b| b == b'\n') {
            ctx.push_line(String::from_utf8_lossy(line).into_owned());
        }
    }
}
