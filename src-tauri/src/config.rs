use serde::{Deserialize, Serialize};

use crate::detector;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct LampConnection {
    pub lamp_id: Option<String>,
    pub lamp_key: Option<String>,
    pub lamp_ip: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Config {
    #[serde(default)]
    pub lamp_connection: LampConnection,
    #[serde(default)]
    pub lamp_tuning: LampTuning,
    #[serde(default)]
    pub monitor_device_name: Option<String>,
    #[serde(default = "default_capture_regions")]
    pub capture_regions: Vec<detector::CaptureRegion>,
    /// Monotonic revision bumped on every persisted change. Not serialized.
    #[serde(skip, default)]
    pub revision: u64,
}

fn default_capture_regions() -> Vec<detector::CaptureRegion> {
    vec![detector::CaptureRegion::FULL]
}

impl Default for Config {
    fn default() -> Self {
        Self {
            lamp_connection: LampConnection::default(),
            lamp_tuning: LampTuning::default(),
            monitor_device_name: None,
            capture_regions: default_capture_regions(),
            revision: 0,
        }
    }
}
