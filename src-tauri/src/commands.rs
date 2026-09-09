use serde::{Deserialize, Serialize};
use std::sync::Arc;
use windows_capture::monitor::Monitor;

use crate::{
    app_store::save_config,
    config::{LampConnection, LampTuning},
    AppState, Config,
};

#[derive(Deserialize, Serialize)]
pub struct LampDeviceInfo {
    #[serde(rename = "lampId")]
    pub lamp_id: String,

    #[serde(rename = "lampIp")]
    pub lamp_ip: String,

    #[serde(rename = "lampKey")]
    pub lamp_key: String,
}

#[derive(Deserialize, Serialize)]
pub struct MonitorInfo {
    pub device_name: String,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub refresh_rate: u32,
}

#[tauri::command]
pub fn get_monitors() -> Result<Vec<MonitorInfo>, String> {
    Monitor::enumerate()
        .map_err(|error| error.to_string())?
        .into_iter()
        .map(|monitor| {
            Ok(MonitorInfo {
                device_name: monitor.device_name().map_err(|error| error.to_string())?,
                name: monitor.name().map_err(|error| error.to_string())?,
                width: monitor.width().map_err(|error| error.to_string())?,
                height: monitor.height().map_err(|error| error.to_string())?,
                refresh_rate: monitor.refresh_rate().map_err(|error| error.to_string())?,
            })
        })
        .collect()
}

#[tauri::command]
pub fn set_monitor(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    device_name: String,
) -> Result<(), String> {
    let monitor_exists = Monitor::enumerate()
        .map_err(|error| error.to_string())?
        .into_iter()
        .any(|monitor| {
            monitor
                .device_name()
                .map(|name| name == device_name)
                .unwrap_or(false)
        });

    if !monitor_exists {
        return Err("Monitor no longer exists".to_string());
    }

    let new_config = {
        let mut config = state.config.write().unwrap();
        config.monitor_device_name = Some(device_name.clone());
        config.clone()
    };

    state
        .monitor_sender
        .send(device_name)
        .map_err(|error| error.to_string())?;

    save_config(&app_handle, &new_config)
}

#[tauri::command]
pub fn connect_lamp(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    lamp: LampDeviceInfo,
) -> Result<(), String> {
    // update the app state
    let new_config = {
        let mut current_config = state.config.write().unwrap();

        let new_lamp_connection = LampConnection {
            lamp_id: Some(lamp.lamp_id),
            lamp_ip: Some(lamp.lamp_ip),
            lamp_key: Some(lamp.lamp_key),
        };

        let new_config = Config {
            lamp_connection: new_lamp_connection.clone(),
            lamp_tuning: current_config.lamp_tuning.clone(),
            monitor_device_name: current_config.monitor_device_name.clone(),
        };

        *current_config = new_config.clone();
        new_config.clone()
    };

    // Send the config to other threads (listeners)
    {
        let (lock, condvar) = &*state.mailbox;
        lock.lock().unwrap().connection_changed = Some(new_config.lamp_connection.clone());
        condvar.notify_one();
    }

    save_config(&app_handle, &new_config)?;

    Ok(())
}

#[tauri::command]
pub fn get_device_info(state: tauri::State<'_, Arc<AppState>>) -> LampDeviceInfo {
    let config = state.config.read().unwrap();
    LampDeviceInfo {
        lamp_id: config.lamp_connection.lamp_id.clone().unwrap_or_default(),
        lamp_ip: config.lamp_connection.lamp_ip.clone().unwrap_or_default(),
        lamp_key: config.lamp_connection.lamp_key.clone().unwrap_or_default(),
    }
}

#[tauri::command]
pub fn get_config(state: tauri::State<'_, Arc<AppState>>) -> Config {
    state.config.read().unwrap().clone()
}

#[tauri::command]
pub fn update_lamp_tuning(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    lamp_tuning: LampTuning,
) -> Result<(), String> {
    let new_config = {
        let mut current_config = state.config.write().unwrap();
        current_config.lamp_tuning = lamp_tuning.clone();
        current_config.clone()
    };

    {
        let (lock, condvar) = &*state.mailbox;
        lock.lock().unwrap().lamp_tuning_changed = Some(lamp_tuning);
        condvar.notify_one();
    }

    save_config(&app_handle, &new_config)
}
