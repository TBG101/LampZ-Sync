// app_store.rs

use crate::Config;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

const STORE_FILE: &str = "store.json";
const CONFIG_KEY: &str = "config";

pub fn load_config(app: &AppHandle) -> Config {
    let store = app.store(STORE_FILE).unwrap();

    store
        .get(CONFIG_KEY)
        .and_then(|value| serde_json::from_value(value).ok())
        .unwrap_or_default()
}

pub fn save_config(app: &AppHandle, config: &Config) -> Result<(), String> {
    let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;

    let value = serde_json::to_value(config).map_err(|e| e.to_string())?;

    store.set(CONFIG_KEY, value);

    store.save().map_err(|e| e.to_string())
}
