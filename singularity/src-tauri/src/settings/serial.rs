use crate::AppData;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Default, Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionType {
    /// Native USB CDC ACM; the line coding baud rate is ignored by the device.
    #[default]
    Usb,
    /// Real UART (or UART bridge) where the baud rate matters.
    Serial,
}

#[derive(Default, Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    /// Line-based text.
    #[default]
    Ascii,
    /// COBS-framed postcard messages.
    Qcp,
}

/// Baud rate handed to the OS for USB CDC ACM ports, which don't use it.
pub const USB_BAUD_RATE: u32 = 115_200;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
pub struct SerialSettings {
    #[serde(default)]
    pub connection_type: ConnectionType,
    #[serde(default)]
    pub protocol: Protocol,
    pub baud_rate: u32,
}

impl Default for SerialSettings {
    fn default() -> Self {
        Self {
            connection_type: ConnectionType::default(),
            protocol: Protocol::default(),
            baud_rate: USB_BAUD_RATE,
        }
    }
}

impl SerialSettings {
    /// Baud rate to open the port with for the selected connection type.
    pub fn effective_baud_rate(&self) -> u32 {
        match self.connection_type {
            ConnectionType::Usb => USB_BAUD_RATE,
            ConnectionType::Serial => self.baud_rate,
        }
    }
}

#[tauri::command]
pub async fn get_serial_settings(
    state: tauri::State<'_, AppData>,
) -> Result<SerialSettings, String> {
    Ok(state.serial.lock().unwrap().settings.clone())
}

#[tauri::command]
pub async fn set_serial_settings(
    settings: SerialSettings,
    state: tauri::State<'_, AppData>,
) -> Result<(), String> {
    state.serial.lock().unwrap().settings = settings;
    Ok(())
}
