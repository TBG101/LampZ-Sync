use std::eprintln;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter, Manager};

use crate::app_state::AppState;
use crate::color::Hsv;
use crate::config::{Config, LampConnection, LampTuning};
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

        let mut connected_with = initial_config.lamp_connection.clone();
        let mut lamp_tune = initial_config.lamp_tuning.clone();
        let mut last_config_revision = initial_config.revision;

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
                apply_config(
                    &snapshot,
                    &mut lamp,
                    &mut connected_with,
                    &mut lamp_tune,
                    &mut interval,
                    &mut timeout,
                );
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
