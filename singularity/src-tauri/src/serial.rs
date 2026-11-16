use std::io::{self, Write};
use std::thread;
use std::time::Duration;

use crate::protocols::{self, Context, ProtocolHandler};
use crate::{settings, AppData};

use serialport::{self, SerialPort};
use tauri::{Emitter, Manager};

#[derive(Default)]
pub struct SerialData {
    pub settings: settings::SerialSettings,
    pub content: Vec<String>,
    pub connected_port: Option<Box<dyn serialport::SerialPort>>,
    pub reconnect_port: Option<String>,
}

#[tauri::command]
pub async fn get_ports() -> Result<Vec<String>, String> {
    let mut port_names: Vec<String> = Vec::new();
    match serialport::available_ports() {
        Ok(ports) => {
            for port in ports {
                if let serialport::SerialPortType::UsbPort(_info) = port.port_type {
                    port_names.push(port.port_name.clone());
                }
            }
        }
        Err(_) => return Err("Error reading ports".to_string()),
    }
    Ok(port_names)
}

fn open_port(
    port: &str,
    settings: &settings::SerialSettings,
) -> serialport::Result<Box<dyn SerialPort>> {
    serialport::new(port, settings.effective_baud_rate())
        .timeout(Duration::from_millis(10))
        .open()
}

#[tauri::command]
pub async fn set_port(port: &str, state: tauri::State<'_, AppData>) -> Result<String, String> {
    let mut serial = state.serial.lock().unwrap();
    serial.connected_port = None;
    serial.reconnect_port = None;
    match open_port(port, &serial.settings) {
        Ok(p) => {
            let name = p.name().unwrap_or("Unknown".to_string());
            serial.connected_port = Some(p);
            serial.reconnect_port = Some(port.to_string());
            Ok(name)
        }
        Err(err) => Err(format!("Failed to connect to {}\n{}", port, err)),
    }
}

#[tauri::command]
pub async fn set_baud_rate(
    baud_rate: u32,
    state: tauri::State<'_, AppData>,
) -> Result<String, String> {
    let mut serial = state.serial.lock().unwrap();
    serial.settings.baud_rate = baud_rate;
    let port = if let Some(p) = &serial.connected_port {
        if let Some(name) = p.name() {
            name
        } else {
            return Ok("".to_string());
        }
    } else {
        return Ok("".to_string());
    };

    serial.connected_port = None;
    match open_port(&port, &serial.settings) {
        Ok(p) => {
            let name = p.name().unwrap_or("Unknown".to_string());
            serial.connected_port = Some(p);
            Ok(name)
        }
        Err(err) => Err(format!("Failed to connect to {}\n{}", port, err)),
    }
}

#[tauri::command]
pub async fn get_active_port(state: tauri::State<'_, AppData>) -> Result<String, String> {
    let serial = state.serial.lock().unwrap();
    if let Some(port) = &serial.connected_port {
        let name = port.name().unwrap_or("".to_string());
        return Ok(name);
    }
    return Ok("".to_string());
}

#[tauri::command]
pub async fn send_serial_message(
    message: String,
    state: tauri::State<'_, AppData>,
) -> Result<String, String> {
    let serial = state.serial.lock().unwrap();

    if let Some(p) = &serial.connected_port {
        let mut port = p.try_clone().expect("Failed to obtain clone");
        let message = message + "\n";
        match port.write(message.as_bytes()) {
            Ok(_) => {
                port.flush().unwrap();
            }
            Err(ref e) => {
                eprintln!("{:?}", e);
                return Err(format!(
                    "Serial port error while trying to send message: {}\n{}",
                    message, e
                ));
            }
        }

        return Ok("".to_string());
    } else {
        return Err("No port connected".to_string());
    }
}

#[tauri::command]
pub async fn get_serial_content(state: tauri::State<'_, AppData>) -> Result<Vec<String>, String> {
    return Ok(state.serial.lock().unwrap().content.clone());
}

#[tauri::command]
pub async fn clear_serial_content(state: tauri::State<'_, AppData>) -> Result<(), String> {
    let mut serial = state.serial.lock().unwrap();
    serial.content = vec!["".to_string()];
    return Ok(());
}

/// Attempt to reopen the last used port if the device is plugged back in
fn try_reconnect(handle: &tauri::AppHandle, serial: &mut SerialData) {
    let Some(name) = serial.reconnect_port.clone() else {
        return;
    };
    let present = serialport::available_ports()
        .map(|ports| ports.iter().any(|p| p.port_name == name))
        .unwrap_or(false);
    if !present {
        return;
    }
    match open_port(&name, &serial.settings) {
        Ok(p) => {
            serial.connected_port = Some(p);
            let _ = handle.emit("serial_reconnected", name);
        }
        Err(e) => eprintln!("Reconnect to {} failed: {}", name, e),
    }
}

/// The handler for the currently selected protocol, tagged so we can tell
/// when the selection changes.
type ActiveHandler = (settings::Protocol, Box<dyn ProtocolHandler>);

pub async fn terminal(handle: &tauri::AppHandle) -> Result<(), String> {
    let mut read_buf = [0u8; 1000];
    let mut active: Option<ActiveHandler> = None;
    loop {
        thread::sleep(Duration::from_millis(100));
        let state = handle.state::<AppData>();
        let mut serial = state.serial.lock().unwrap();

        let Some(mut port) = serial
            .connected_port
            .as_ref()
            .map(|p| p.try_clone().expect("Failed to obtain clone"))
        else {
            active = None; // drop decoder state while disconnected
            try_reconnect(handle, &mut serial);
            continue;
        };

        // (Re)build the handler when the protocol setting changes.
        let wanted = serial.settings.protocol;
        let (_, protocol) = match &mut active {
            Some(current) if current.0 == wanted => current,
            slot => slot.insert((wanted, protocols::handler_for(wanted))),
        };

        if let Err(e) = protocol.poll(port.as_mut()) {
            eprintln!("Protocol poll failed: {}", e);
        }

        match port.read(&mut read_buf) {
            Ok(n) => {
                let mut ctx = Context {
                    app: handle,
                    serial: &mut serial,
                };
                protocol.on_data(&read_buf[..n], &mut ctx);
            }
            Err(e) => match e.kind() {
                io::ErrorKind::TimedOut => {} // expected
                io::ErrorKind::BrokenPipe
                | io::ErrorKind::NotConnected
                | io::ErrorKind::UnexpectedEof => {
                    serial.connected_port = None;
                    let _ = handle.emit("serial_disconnected", ());
                    eprintln!("Port disconnected");
                }
                _ => eprintln!("{:?}", e),
            },
        }
        let _ = port.flush();
    }
}
