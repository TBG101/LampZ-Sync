use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, State};
use ts_rs::TS;
use windows_capture::monitor::Monitor;

use crate::{
    app_state::AppState,
    app_store::save_config,
    config::{Config, LampConnection, LampMode, LampTuning, StaticColor},
    detector::CaptureRegion,
};

#[derive(Deserialize, Serialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
pub struct LampDeviceInfo {
    #[serde(rename = "lampId")]
    pub lamp_id: String,

    #[serde(rename = "lampIp")]
    pub lamp_ip: String,

    #[serde(rename = "lampKey")]
    pub lamp_key: String,
}

#[derive(Deserialize, Serialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
pub struct MonitorInfo {
    pub device_name: String,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub refresh_rate: u32,
}

/// The single place where config is mutated, persisted and published.
///
/// Every command goes through here, so adding a new option means mutating its
/// field once — persistence, the revision bump and the worker wake-up happen
/// automatically. This replaces the old pattern of rebuilding `Config` by hand
/// in `connect_lamp` (which silently reset any field that was forgotten).
fn update_config<F>(app_handle: &AppHandle, state: &AppState, mutate: F) -> Result<(), String>
where
    F: FnOnce(&mut Config),
{
    let (snapshot, epoch) = {
        let mut config = state.config.write().unwrap();
        mutate(&mut config);
        config.revision = config.revision.wrapping_add(1);
        (config.clone(), config.revision)
    };

    {
        let (lock, condvar) = &*state.mailbox;
        lock.lock().unwrap().config_epoch = epoch;
        condvar.notify_all();
    }

    save_config(app_handle, &snapshot)
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
    app_handle: AppHandle,
    state: State<'_, Arc<AppState>>,
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

    update_config(&app_handle, state.inner(), |config| {
        config.monitor_device_name = Some(device_name.clone());
    })?;

    state
        .monitor_sender
        .send(device_name)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn connect_lamp(
    app_handle: AppHandle,
    state: State<'_, Arc<AppState>>,
    lamp: LampDeviceInfo,
) -> Result<(), String> {
    let lamp_connection = LampConnection {
        lamp_id: Some(lamp.lamp_id),
        lamp_ip: Some(lamp.lamp_ip),
        lamp_key: Some(lamp.lamp_key),
    };

    update_config(&app_handle, state.inner(), move |config| {
        config.lamp_connection = lamp_connection;
    })
}

#[tauri::command]
pub fn get_device_info(state: State<'_, Arc<AppState>>) -> LampDeviceInfo {
    let config = state.config.read().unwrap();
    LampDeviceInfo {
        lamp_id: config.lamp_connection.lamp_id.clone().unwrap_or_default(),
        lamp_ip: config.lamp_connection.lamp_ip.clone().unwrap_or_default(),
        lamp_key: config.lamp_connection.lamp_key.clone().unwrap_or_default(),
    }
}

#[tauri::command]
pub fn get_config(state: State<'_, Arc<AppState>>) -> Config {
    state.config.read().unwrap().clone()
}

#[tauri::command]
pub fn update_lamp_tuning(
    app_handle: AppHandle,
    state: State<'_, Arc<AppState>>,
    lamp_tuning: LampTuning,
) -> Result<(), String> {
    update_config(&app_handle, state.inner(), move |config| {
        config.lamp_tuning = lamp_tuning;
    })
}

#[tauri::command]
pub fn update_capture_regions(
    app_handle: AppHandle,
    state: State<'_, Arc<AppState>>,
    capture_regions: Vec<CaptureRegion>,
) -> Result<(), String> {
    update_config(&app_handle, state.inner(), move |config| {
        config.capture_regions = capture_regions;
    })
}

#[tauri::command]
pub fn set_sync_enabled(
    app_handle: AppHandle,
    state: State<'_, Arc<AppState>>,
    enabled: bool,
) -> Result<(), String> {
    update_config(&app_handle, state.inner(), move |config| {
        config.sync_enabled = enabled;
    })
}

#[tauri::command]
pub fn set_lamp_mode(
    app_handle: AppHandle,
    state: State<'_, Arc<AppState>>,
    mode: LampMode,
) -> Result<(), String> {
    update_config(&app_handle, state.inner(), move |config| {
        config.lamp_mode = mode;
    })
}

#[tauri::command]
pub fn set_static_color(
    app_handle: AppHandle,
    state: State<'_, Arc<AppState>>,
    static_color: StaticColor,
) -> Result<(), String> {
    update_config(&app_handle, state.inner(), move |config| {
        config.static_color = StaticColor {
            hex: static_color.hex.to_lowercase(),
            brightness: static_color.brightness,
        };
    })
}
