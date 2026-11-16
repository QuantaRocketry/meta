use crate::AppData;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
#[serde(tag = "type", content = "data")]
pub enum DeviceSettings {
    Entangler(EntanglerSettings),
    Warp(WarpSettings),
    #[serde(rename = "Wio-L1")]
    WioL1(WioL1Settings),
}

#[derive(Default, Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
pub struct EntanglerSettings {
    lora: LoRaSettings,
}

#[derive(Default, Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
pub struct WarpSettings {
    deployment: DeploymentSettings,
}

#[derive(Default, Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
pub struct WioL1Settings {
    lora: LoRaSettings,
}

#[derive(Default, Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
pub struct LoRaSettings {
    frequency: u32,
    bandwidth: u32,
    spreading_factor: u8,
    sync_word: u8,
    coding_rate: u8,
}

#[derive(Default, Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
struct DeploymentSettings {
    apogee: bool,
    main: bool,
    apogee_delay: f32,
    main_altitude: f32,
}

#[tauri::command]
pub async fn get_device_settings(
    state: tauri::State<'_, AppData>,
) -> Result<DeviceSettings, String> {
    let board_settings = state.board_settings.lock().unwrap();
    match board_settings.clone() {
        Some(s) => Ok(s),
        None => Err(format!("No board selected")),
    }
}

#[tauri::command]
pub fn set_device_settings(settings: DeviceSettings, state: tauri::State<'_, AppData>) {
    let mut board_settings = state.board_settings.lock().unwrap();
    *board_settings = Some(settings);
}

#[tauri::command]
pub async fn upload_device_settings(
    _settings: DeviceSettings,
    state: tauri::State<'_, AppData>,
) -> Result<(), String> {
    let serial = state.serial.lock().unwrap();
    if serial.connected_port.is_none() {
        return Err(format!("No device connected"));
    }

    return Err(format!("Upload not implemented"));
}

#[tauri::command]
pub async fn download_device_settings(
    state: tauri::State<'_, AppData>,
) -> Result<DeviceSettings, String> {
    let serial = state.serial.lock().unwrap();
    if serial.connected_port.is_none() {
        return Err(format!("No device connected"));
    }

    let mut board_settings = state.board_settings.lock().unwrap();
    *board_settings = Some(DeviceSettings::Entangler(EntanglerSettings::default()));
    eprintln!("Attempted to download settings. Not implemented");
    return Ok(board_settings.unwrap());
}

#[tauri::command]
pub fn get_device_variants() -> Vec<DeviceSettings> {
    vec![
        DeviceSettings::Entangler(EntanglerSettings::default()),
        DeviceSettings::Warp(WarpSettings::default()),
        DeviceSettings::WioL1(WioL1Settings::default()),
    ]
}

#[tauri::command]
pub async fn set_device_variant(
    device: String,
    state: tauri::State<'_, AppData>,
) -> Result<(), String> {
    let settings = match device.as_str() {
        "Entangler" => DeviceSettings::Entangler(EntanglerSettings::default()),
        "Warp" => DeviceSettings::Warp(WarpSettings::default()),
        "Wio-L1" => DeviceSettings::WioL1(WioL1Settings::default()),
        _ => return Err(format!("No matching device in backend: {}", device)),
    };

    let mut board_settings = state.board_settings.lock().unwrap();
    *board_settings = Some(settings);
    Ok(())
}
