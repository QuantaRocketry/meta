// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod odometry;
mod protocols;
mod serial;
mod settings;

use std::fs;
use std::sync::Mutex;
use tauri::Manager;

use crate::odometry::OdometryData;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
async fn greet(name: &str) -> Result<String, String> {
    Ok(format!("Hello, {}! You've been greeted from Rust!", name))
}

#[derive(Default)]
struct AppData {
    pub app_settings: Mutex<settings::AppSettings>,
    pub board_settings: Mutex<Option<settings::DeviceSettings>>,
    pub serial: Mutex<serial::SerialData>,
    pub odometry: OdometryData,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_os::init())
        .setup(|app| {
            let handler_clone = app.handle().clone();

            let config_dir = app.path().app_config_dir().unwrap();
            if let Err(e) = fs::create_dir_all(&config_dir) {
                eprintln!("Failed to create config dir: {}", e);
            };

            let path = config_dir.join(settings::FILE_NAME);
            let app_settings = settings::AppSettings::load(path);

            let app_data = AppData::default();
            *app_data.app_settings.lock().unwrap() = app_settings;

            app.manage(app_data);
            tauri::async_runtime::spawn(async move {
                if let Err(e) = serial::terminal(&handler_clone).await {
                    eprintln!("Error in terminal: {}", e);
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            serial::set_port,
            serial::set_baud_rate,
            serial::get_ports,
            serial::get_active_port,
            serial::get_serial_content,
            serial::send_serial_message,
            serial::clear_serial_content,
            serial::set_baud_rate,
            settings::app::get_cesium_ion_token,
            settings::app::set_cesium_ion_token,
            settings::app::get_app_settings,
            settings::app::set_app_settings,
            settings::device::upload_device_settings,
            settings::device::download_device_settings,
            settings::device::get_device_settings,
            settings::device::set_device_settings,
            settings::device::set_device_variant,
            settings::device::get_device_variants,
            settings::serial::set_serial_settings,
            settings::serial::get_serial_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
