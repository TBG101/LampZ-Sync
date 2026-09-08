use std::sync::Arc;
use std::time::{Duration, Instant};
use std::eprintln;

use tauri::{AppHandle, Emitter, Manager};

use crate::color::Hsv;
use crate::{lamp, LampConnection};
use crate::{AppState, LampTuning};

enum LampEvent {
    LampConnectionChanged(LampConnection),
    Color(crate::color::Rgb),
    TuningChanged(LampTuning),
}

fn connect_lamp(lamp_connection: &LampConnection) -> Result<lamp::Lamp, String> {
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

pub fn start_lamp_thread(app: &AppHandle) {
    let app_handle = app.clone();

    std::thread::spawn(move || {
        let app_handle = app_handle.clone();
        let state = app_handle.state::<Arc<AppState>>();

        let initial_config = state.config.read().unwrap().clone();

        let mut lamp = match connect_lamp(&initial_config.lamp_connection) {
            Ok(lamp) => Some(lamp),
            Err(error) => {
                eprintln!("Initial lamp connection failed: {error}");
                None
            }
        };

        let mut lamp_tune = initial_config.lamp_tuning.clone();

        let mut interval = Duration::from_millis(lamp_tune.poll_interval_ms);
        let mut last_update = Instant::now() - interval;

        let mut timeout = Duration::from_secs(lamp_tune.mailbox_timeout_secs);
        let mut last_connected = false;
        let mut last_hsv_sent: Option<Hsv> = None;
        let mut last_sent_v = 0;

        loop {
            let elapsed = last_update.elapsed();

            if elapsed < interval {
                std::thread::sleep(interval - elapsed);
            }

            let event = {
                let (lock, condvar) = &*state.mailbox;
                let mut mailbox = lock.lock().unwrap();

                while mailbox.connection_changed.is_none()
                    && mailbox.latest_color.is_none()
                    && mailbox.lamp_tuning_changed.is_none()
                {
                    let (guard, _) = condvar.wait_timeout(mailbox, timeout).unwrap();
                    mailbox = guard;

                    if mailbox.connection_changed.is_none() && mailbox.latest_color.is_none() {
                        break;
                    }
                }

                match (
                    mailbox.connection_changed.take(),
                    mailbox.latest_color.take(),
                    mailbox.lamp_tuning_changed.take(),
                ) {
                    (Some(lamp_connection), _, _) => {
                        Some(LampEvent::LampConnectionChanged(lamp_connection))
                    }
                    (None, _, Some(lamp_tuning)) => Some(LampEvent::TuningChanged(lamp_tuning)),
                    (None, Some(color), None) => Some(LampEvent::Color(color)),
                    (None, None, None) => None,
                }
            };

            if let Some(current_lamp) = lamp.as_ref() {
                let connected = current_lamp.is_connected();

                if connected != last_connected {
                    last_connected = connected;
                    app_handle
                        .clone()
                        .emit("lamp-connection", last_connected)
                        .unwrap()
                }
            }

            let Some(event) = event else {
                continue;
            };

            match event {
                LampEvent::LampConnectionChanged(config) => {
                    lamp = match connect_lamp(&config) {
                        Ok(lamp) => Some(lamp),
                        Err(error) => {
                            eprintln!("Initial lamp connection failed: {error}");
                            None
                        }
                    };
                }

                LampEvent::TuningChanged(lamp_tuning) => {
                    lamp_tune = lamp_tuning;

                    interval = Duration::from_millis(lamp_tune.poll_interval_ms);
                    timeout = Duration::from_secs(lamp_tune.mailbox_timeout_secs);
                }

                LampEvent::Color(color) => {
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
            }
        }
    });
}
