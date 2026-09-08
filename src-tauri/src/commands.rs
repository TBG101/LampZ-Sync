use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{app_store::save_config, AppState, Config, LampConnection, LampTuning};

#[derive(Deserialize, Serialize)]
pub struct LampDeviceInfo {
    #[serde(rename = "lampId")]
    pub lamp_id: String,

    #[serde(rename = "lampIp")]
    pub lamp_ip: String,

    #[serde(rename = "lampKey")]
    pub lamp_key: String,
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
