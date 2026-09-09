use serde::{Deserialize, Serialize};

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


#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Config {
    pub lamp_connection: LampConnection,
    pub lamp_tuning: LampTuning,
    pub monitor_device_name: Option<String>,
}
