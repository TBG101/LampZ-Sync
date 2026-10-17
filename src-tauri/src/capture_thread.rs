use std::sync::Arc;
use std::{
    sync::mpsc::Receiver,
    time::{Duration, Instant},
};

use windows_capture::monitor::Monitor;

use crate::app_state::AppState;
use crate::screen_capture::ScreenCapture;
use crate::detector::detect_color;

fn find_monitor(m_device_name: &str) -> Monitor {
    let monitors = Monitor::enumerate().unwrap();

    monitors
        .into_iter()
        .find(|m| m.device_name().unwrap() == m_device_name)
        .or_else(|| Monitor::enumerate().unwrap().into_iter().next())
        .expect("No monitors available")
}

pub fn start_capture_thread(
    state: Arc<AppState>,
    monitor_rx: Receiver<String>,
    initial_monitor_device_name: Option<String>,
) {
    std::thread::spawn(move || {
        let mut regions = state.config.read().unwrap().capture_regions.clone();
        let mut sync_enabled = state.config.read().unwrap().sync_enabled;
        let mut last_config_revision = state.config.read().unwrap().revision;

        let mut current_monitor_device_name = match initial_monitor_device_name {
            Some(device_name) => find_monitor(&device_name).device_name().unwrap(),
            None => Monitor::primary()
                .unwrap()
                .device_name()
                .expect("No monitors available"),
        };

        let mut screen_capture =
            match ScreenCapture::new(find_monitor(&current_monitor_device_name).index().unwrap()) {
                Ok(capture) => capture,
                Err(e) => {
                    eprintln!("Failed to initialize screen capture: {}", e);
                    return;
                }
            };

        let mut frames = 0u64;
        let mut capture_time = Duration::ZERO;
        let mut detect_time = Duration::ZERO;
        let mut last_report = Instant::now();

        loop {
            // Drain pending changes and keep only the newest one.
            if let Some(requested_monitor_d_name) = monitor_rx.try_iter().last() {
                if requested_monitor_d_name != current_monitor_device_name {
                    let selected_m = find_monitor(&requested_monitor_d_name);

                    match ScreenCapture::new(selected_m.index().unwrap()) {
                        Ok(new_capture) => {
                            screen_capture = new_capture;
                            current_monitor_device_name = requested_monitor_d_name;
                            println!("Switched monitor");
                        }
                        Err(error) => {
                            eprintln!("Failed to switch monitor: {error}");
                        }
                    }
                }
            }

            // Regions come straight from the single source of truth.
            {
                let snapshot = state.config.read().unwrap();
                if snapshot.revision != last_config_revision {
                    last_config_revision = snapshot.revision;
                    regions = snapshot.capture_regions.clone();
                    sync_enabled = snapshot.sync_enabled;
                }
            }

            // Sync disabled: skip capture/detection entirely. Pacing here is
            // driven by frame arrival, so sleep to avoid a busy spin.
            if !sync_enabled {
                std::thread::sleep(Duration::from_millis(50));
                continue;
            }

            let frame_start = Instant::now();

            let _ = screen_capture.process_frame(|buffer, width, height| {
                let start = Instant::now();

                let rgb = match detect_color(buffer, width, height, &regions) {
                    Some(value) => value,
                    None => return,
                };

                detect_time += start.elapsed();

                let start = Instant::now();

                // Send the detected color to the lamp thread
                let (lock, condvar) = &*state.mailbox;
                lock.lock().unwrap().latest_color = Some(rgb);
                condvar.notify_one();

                capture_time += start.elapsed();

                frames += 1;
            });

            let frame_time = frame_start.elapsed();

            if last_report.elapsed() >= Duration::from_secs(1) {
                println!(
                    "FPS: {:.1} | frame: {:.2}ms | detect: {:.2}ms | send: {:.3}ms",
                    frames as f64 / last_report.elapsed().as_secs_f64(),
                    frame_time.as_secs_f64() * 1000.0,
                    detect_time.as_secs_f64() * 1000.0 / frames.max(1) as f64,
                    capture_time.as_secs_f64() * 1000.0 / frames.max(1) as f64,
                );

                frames = 0;
                capture_time = Duration::ZERO;
                detect_time = Duration::ZERO;
                last_report = Instant::now();
            }
        }
    });
}
