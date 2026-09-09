mod app_state;
mod app_store;
mod capture_thread;
mod color;
mod commands;
mod config;
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
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

use crate::{
    app_state::{AppState, LampMailbox}, color::Rgb, commands::{
        connect_lamp, get_config, get_device_info, get_monitors, set_monitor, update_lamp_tuning,
    }, config::Config,
};

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
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .setup(move |app| {
            // Load persisted config
            let config = app_store::load_config(&app.handle());

            // Put persisted config into runtime state
            {
                let mut state_config = state.config.write().unwrap();
                *state_config = config.clone();
            }

            // -------------------------
            // Create tray menu
            // -------------------------

            let show = MenuItem::with_id(app, "show", "Show LampZ Sync", true, None::<&str>)?;

            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

            let menu = Menu::with_items(app, &[&show, &quit])?;

            // -------------------------
            // Create tray icon
            // -------------------------

            TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }

                    "quit" => {
                        app.exit(0);
                    }

                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();

                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            // Start workers after config has been loaded
            start_lampz_sync(
                Arc::clone(&state),
                &app.handle(),
                monitor_rx,
                config.monitor_device_name.clone(),
            );

            if let Some(window) = app.get_webview_window("main") {
                let app_handle = app.handle().clone();
                let window_for_event = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();

                        let _ = window_for_event.hide();
                    }
                });
            }

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
