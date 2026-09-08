use crate::SharedMailbox;
use crate::screen_capture::ScreenCapture;
use crate::{detector::detect_color, LampMailbox};
use std::time::{Duration, Instant};

pub fn start_capture_thread(mail_box: SharedMailbox) {
    std::thread::spawn(move || {
        let mut screen_capture = match ScreenCapture::new(2) {
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
            let frame_start = Instant::now();

            let _ = screen_capture.process_frame(|buffer, width, height| {
                let start = Instant::now();

                let rgb = match detect_color(buffer, width, height) {
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
