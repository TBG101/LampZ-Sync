use std::eprintln;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter, Manager};

use crate::app_state::AppState;
use crate::color::Hsv;
use crate::config::{Config, LampConnection, LampMode, LampTuning, StaticColor};
use crate::lamp;

fn open_lamp(lamp_connection: &LampConnection) -> Result<lamp::Lamp, String> {
    let device_id = lamp_connection
        .lamp_id
        .as_deref()
        .ok_or("Lamp ID is missing")?;
    let ip = lamp_connection
        .lamp_ip
        .as_deref()
        .ok_or("Lamp IP is missing")?;
    let key = lamp_connection
        .lamp_key
        .as_deref()
        .ok_or("Lamp key is missing")?;

    let lamp = lamp::Lamp::new(device_id, ip, key).map_err(|error| error.to_string())?;

    lamp.on().map_err(|error| error.to_string())?;
    lamp.colour_mode().map_err(|error| error.to_string())?;

    Ok(lamp)
}

fn connect_or_log(lamp_connection: &LampConnection) -> Option<lamp::Lamp> {
    match open_lamp(lamp_connection) {
        Ok(lamp) => Some(lamp),
        Err(error) => {
            eprintln!("Lamp connection failed: {error}");
            None
        }
    }
}

/// Pushes a fixed color to the lamp. The brightness replaces the value
/// component so the same hue can be dimmed without changing the swatch.
fn apply_static_color(lamp: &Option<lamp::Lamp>, color: &StaticColor) {
    let Some(current_lamp) = lamp.as_ref() else {
        return;
    };

    let Some(hsv) = lamp::hex_to_hsv(&color.hex) else {
        eprintln!("Invalid static color hex: {}", color.hex);
        return;
    };

    if let Err(error) = current_lamp.set_hsv(hsv.h, hsv.s, color.brightness) {
        eprintln!("Failed to set static color: {error}");
    }
}

/// Applies a fresh config snapshot to the lamp thread's local state.
///
/// Diffing against the last-seen values means a new *tuning* field works with
/// no changes here (the whole `LampTuning` is swapped in); only brand-new
/// config sections need an extra branch.
fn apply_config(
    config: &Config,
    lamp: &mut Option<lamp::Lamp>,
    connected_with: &mut LampConnection,
    lamp_tune: &mut LampTuning,
    interval: &mut Duration,
    timeout: &mut Duration,
) {
    if config.lamp_connection != *connected_with {
        *connected_with = config.lamp_connection.clone();
        *lamp = connect_or_log(&config.lamp_connection);
    }

    if config.lamp_tuning != *lamp_tune {
        *lamp_tune = config.lamp_tuning.clone();
        *interval = Duration::from_millis(lamp_tune.poll_interval_ms);
        *timeout = Duration::from_secs(lamp_tune.mailbox_timeout_secs);
    }
}

pub fn start_lamp_thread(app: &AppHandle) {
    let app_handle = app.clone();

    std::thread::spawn(move || {
        let app_handle = app_handle.clone();
        let state = app_handle.state::<Arc<AppState>>();

        let initial_config = state.config.read().unwrap().clone();

        let mut lamp = connect_or_log(&initial_config.lamp_connection);

        if !initial_config.powered {
            if let Some(current_lamp) = lamp.as_ref() {
                let _ = current_lamp.off();
            }
        } else if initial_config.lamp_mode == LampMode::Static {
            apply_static_color(&lamp, &initial_config.static_color);
        }

        let mut connected_with = initial_config.lamp_connection.clone();
        let mut lamp_tune = initial_config.lamp_tuning.clone();
        let mut last_config_revision = initial_config.revision;
        let mut last_powered = initial_config.powered;

        let mut interval = Duration::from_millis(lamp_tune.poll_interval_ms);
        let mut timeout = Duration::from_secs(lamp_tune.mailbox_timeout_secs);
        let mut last_update = Instant::now() - interval;

        let mut last_connected = false;
        let mut last_hsv_sent: Option<Hsv> = None;
        let mut last_sent_v = 0;

        loop {
            // Pull the single source of truth. Any persisted change shows up as
            // a higher revision, so nothing can be silently missed.
            let snapshot = state.config.read().unwrap().clone();
            if snapshot.revision != last_config_revision {
                last_config_revision = snapshot.revision;

                let reconnected = snapshot.lamp_connection != connected_with;

                apply_config(
                    &snapshot,
                    &mut lamp,
                    &mut connected_with,
                    &mut lamp_tune,
                    &mut interval,
                    &mut timeout,
                );

                // Apply power transitions. A reconnect also powers the lamp on
                // (see `open_lamp`), so reconcile when it is meant to be off.
                if snapshot.powered != last_powered || (reconnected && !snapshot.powered) {
                    last_powered = snapshot.powered;

                    if let Some(current_lamp) = lamp.as_ref() {
                        let result = if snapshot.powered {
                            current_lamp.on()
                        } else {
                            current_lamp.off()
                        };

                        if let Err(error) = result {
                            eprintln!("Failed to set lamp power: {error}");
                        }
                    }

                    // Force an immediate color push when powering back on.
                    last_hsv_sent = None;
                    last_sent_v = 0;
                }

                // Re-push on every change so a reconnect or color edit lands.
                if snapshot.lamp_mode == LampMode::Static {
                    apply_static_color(&lamp, &snapshot.static_color);
                }
            }

            let elapsed = last_update.elapsed();

            if elapsed < interval {
                std::thread::sleep(interval - elapsed);
            }

            let color = {
                let (lock, condvar) = &*state.mailbox;
                let mut mailbox = lock.lock().unwrap();

                while mailbox.latest_color.is_none() && mailbox.config_epoch <= last_config_revision {
                    let (guard, _) = condvar.wait_timeout(mailbox, timeout).unwrap();
                    mailbox = guard;

                    if mailbox.latest_color.is_none() && mailbox.config_epoch <= last_config_revision {
                        break;
                    }
                }

                mailbox.latest_color.take()
            };

            if let Some(current_lamp) = lamp.as_ref() {
                let connected = current_lamp.is_connected();

                if connected != last_connected {
                    last_connected = connected;
                    let _ = app_handle.clone().emit("lamp-connection", last_connected);
                }
            }

            // Powered off, sync disabled, or in static mode: stop pushing
            // colors. Reset the diff state so resuming forces an update.
            if !snapshot.powered || !snapshot.sync_enabled || snapshot.lamp_mode == LampMode::Static {
                last_hsv_sent = None;
                last_sent_v = 0;
                continue;
            }

            let Some(color) = color else {
                continue;
            };

            let Some(current_lamp) = lamp.as_ref() else {
                continue;
            };

            let hsv = color.rgb_to_hsv();

            let v = hsv.v as f32 / 1000.0;
            let v_final = (v.powf(lamp_tune.gamma) * 1000.0).clamp(0.0, 1000.0) as u16;

            let should_update = if let Some(last_hsv) = last_hsv_sent.as_ref() {
                hsv.hue_changed(last_hsv, lamp_tune.hue_threshold)
                    || hsv.saturation_changed(last_hsv, lamp_tune.sat_threshold)
                    || v_final.abs_diff(last_sent_v) >= lamp_tune.v_threshold
            } else {
                true
            };

            if should_update {
                if let Err(e) = current_lamp.set_hsv(hsv.h, hsv.s, v_final) {
                    eprintln!("Failed to set lamp color: {}", e);
                } else {
                    last_hsv_sent = Some(hsv);
                    last_sent_v = v_final;
                    last_update = Instant::now();
                }
            }
        }
    });
}
