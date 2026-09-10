use crate::detector::CaptureRegion;
use crate::screen_capture::ScreenCapture;
use crate::{app_state::SharedMailbox, detector::detect_color};
use std::{
    sync::mpsc::Receiver,
    time::{Duration, Instant},
};
use windows_capture::monitor::Monitor;

fn find_monitor(m_device_name: &str) -> Monitor {
    let monitors = Monitor::enumerate().unwrap();

    monitors
        .into_iter()
        .find(|m| m.device_name().unwrap() == m_device_name)
        .or_else(|| Monitor::enumerate().unwrap().into_iter().next())
        .expect("No monitors available")
}

pub fn start_capture_thread(
    mail_box: SharedMailbox,
    monitor_rx: Receiver<String>,
    initial_monitor_device_name: Option<String>,
    initual_capture_region: Vec<CaptureRegion>,
) {
    std::thread::spawn(move || {
        let mut regions = initual_capture_region;
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

            let (lock, _) = &*mail_box;

            if let Some(new_regions) = lock.lock().unwrap().region_changed.take() {
                regions = new_regions;
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
                let (lock, condvar) = &*mail_box;
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
