mod app_store;
mod capture_thread;
mod color;
mod commands;
mod detector;
mod lamp;
mod lamp_thread;
mod screen_capture;

use serde::{Deserialize, Serialize};
use std::sync::{
    mpsc::{self, Receiver},
    Arc, Condvar, Mutex, RwLock,
};
use tauri::AppHandle;

use crate::{
    color::Rgb,
    commands::{
        connect_lamp, get_config, get_device_info, get_monitors, set_monitor,
        update_lamp_tuning,
    },
};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct LampTuning {
    pub poll_interval_ms: u64,
    pub mailbox_timeout_secs: u64,
    pub gamma: f32,         // the 2.0 in v.powf(2.0)
    pub hue_threshold: u16, // the 2 in hue_changed
    pub sat_threshold: u16, // the 10 in saturation_changed
    pub v_threshold: u16,   // the 10 in v_final.abs_diff(...)
}

impl Default for LampTuning {
    fn default() -> Self {
        Self {
            poll_interval_ms: 50,
            mailbox_timeout_secs: 1,
            gamma: 2.0,
            hue_threshold: 2,
            sat_threshold: 10,
            v_threshold: 10,
        }
    }
}

#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct LampConnection {
    pub lamp_id: Option<String>,
    pub lamp_key: Option<String>,
    pub lamp_ip: Option<String>,
}

pub struct LampMailbox {
    pub latest_color: Option<Rgb>,
    pub connection_changed: Option<LampConnection>,
    pub lamp_tuning_changed: Option<LampTuning>,
}

pub type SharedMailbox = Arc<(Mutex<LampMailbox>, Condvar)>;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Config {
    pub lamp_connection: LampConnection,
    pub lamp_tuning: LampTuning,
    pub monitor_device_name: Option<String>,
}

pub struct AppState {
    pub config: RwLock<Config>,
    pub mailbox: SharedMailbox,
    pub monitor_sender: std::sync::mpsc::Sender<String>,
}

fn start_lampz_sync(
    state: Arc<AppState>,
    app: &AppHandle,
    monitor_sender: Receiver<String>,
    inital_device_name: Option<String>,
) {
    let mailbox = Arc::clone(&state.mailbox);

    capture_thread::start_capture_thread(Arc::clone(&mailbox), monitor_sender, inital_device_name);

    lamp_thread::start_lamp_thread(&app.clone());
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mailbox = Arc::new((
        Mutex::new(LampMailbox {
            latest_color: None,
            connection_changed: None,
            lamp_tuning_changed: None,
        }),
        Condvar::new(),
    ));

    let (monitor_tx, monitor_rx) = mpsc::channel::<String>();

    let state = Arc::new(AppState {
        config: RwLock::new(Config::default()),
        mailbox: mailbox,
        monitor_sender: monitor_tx,
    });

    tauri::Builder::default()
        .manage(Arc::clone(&state))
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .setup(move |app| {
            // Load persisted config
            let config = app_store::load_config(&app.handle());

            // Put persisted config into runtime state
            {
                let mut state_config = state.config.write().unwrap();
                *state_config = config.clone();
            }

            // Start workers after config has been loaded
            start_lampz_sync(
                Arc::clone(&state),
                &app.handle(),
                monitor_rx,
                config.monitor_device_name.clone(),
            );

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            connect_lamp,
            get_config,
            get_device_info,
            get_monitors,
            set_monitor,
            update_lamp_tuning
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
