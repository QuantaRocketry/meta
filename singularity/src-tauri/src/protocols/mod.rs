//! Wire protocols spoken over the serial port.
//!
//! # Adding a protocol
//!
//! 1. Add a variant to [`settings::Protocol`](crate::settings::Protocol).
//! 2. Create `protocols/<name>.rs` with a struct implementing [`ProtocolHandler`].
//! 3. Declare the module below and add an arm to [`handler_for`].
//!
//! The compiler will flag the `match` in [`handler_for`] until step 3 is done.
//! The terminal needs no changes.

mod ascii;
mod qcp;

use serialport::SerialPort;
use tauri::{AppHandle, Emitter};

use crate::serial::SerialData;
use crate::settings::Protocol;

/// What a handler can act on when data arrives: the app (for emitting events)
/// and the shared serial state (for the terminal log).
pub struct Context<'a> {
    pub app: &'a AppHandle,
    pub serial: &'a mut SerialData,
}

impl Context<'_> {
    /// Append a line to the terminal log and notify the frontend.
    pub fn push_line(&mut self, line: String) {
        let _ = self.app.emit("serial_message_received", line.clone());
        self.serial.content.push(line);
    }
}

/// Per-protocol behaviour driven by the terminal loop.
///
/// A fresh handler is created whenever the selected protocol changes or the
/// port disconnects, so implementations can keep decoding state (partial
/// frames, poll timers, ...) in `self` without worrying about stale data.
pub trait ProtocolHandler: Send {
    /// Called every monitor tick before reading. Use it to send periodic
    /// requests. Errors are logged and otherwise ignored.
    fn poll(&mut self, _port: &mut dyn SerialPort) -> Result<(), String> {
        Ok(())
    }

    /// Called with every chunk of bytes read from the port. What happens next
    /// is up to the protocol: log lines, emit events, update state, or nothing.
    fn on_data(&mut self, bytes: &[u8], ctx: &mut Context);
}

/// Build the handler for `protocol`.
pub fn handler_for(protocol: Protocol) -> Box<dyn ProtocolHandler> {
    match protocol {
        Protocol::Ascii => Box::new(ascii::Ascii),
        Protocol::Qcp => Box::new(qcp::Qcp::default()),
    }
}
