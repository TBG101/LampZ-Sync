use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::detector;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
pub struct LampTuning {
    #[ts(type = "number")]
    pub poll_interval_ms: u64,
    #[ts(type = "number")]
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

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
pub struct LampConnection {
    pub lamp_id: Option<String>,
    pub lamp_key: Option<String>,
    pub lamp_ip: Option<String>,
}

/// What drives the lamp: the sampled screen color or a fixed static color.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../src/lib/bindings/")]
pub enum LampMode {
    Sync,
    Static,
}

impl Default for LampMode {
    fn default() -> Self {
        Self::Sync
    }
}

/// A fixed lamp color. `hex` is `#RRGGBB`; `brightness` overrides the value
/// component (`0..=1000`) so the same hue can be dimmed independently.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
pub struct StaticColor {
    pub hex: String,
    #[ts(type = "number")]
    pub brightness: u16,
}

impl Default for StaticColor {
    fn default() -> Self {
        Self {
            hex: "#ffffff".to_string(),
            brightness: 1000,
        }
    }
}

/// Local MCP (Model Context Protocol) server used by external agents to drive
/// the lamp. Binds to `127.0.0.1` only.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
pub struct McpConfig {
    /// Whether the embedded MCP server is running.
    pub enabled: bool,
    /// TCP port the Streamable HTTP endpoint listens on.
    pub port: u16,
    /// Optional bearer token. Empty/`None` leaves the endpoint unauthenticated.
    pub token: Option<String>,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            port: 7331,
            token: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
pub struct Config {
    #[serde(default)]
    pub lamp_connection: LampConnection,
    #[serde(default)]
    pub lamp_tuning: LampTuning,
    #[serde(default)]
    pub monitor_device_name: Option<String>,
    #[serde(default = "default_capture_regions")]
    pub capture_regions: Vec<detector::CaptureRegion>,
    /// Whether screen capture/detection and lamp updates are running.
    #[serde(default = "default_sync_enabled")]
    pub sync_enabled: bool,
    /// Whether the lamp follows the screen or shows a fixed static color.
    #[serde(default)]
    pub lamp_mode: LampMode,
    /// The fixed color used when `lamp_mode` is `Static`.
    #[serde(default)]
    pub static_color: StaticColor,
    /// Master power for the lamp. When off, no colors are pushed and the lamp is
    /// turned off at the driver level.
    #[serde(default = "default_powered")]
    pub powered: bool,
    /// Embedded MCP server settings for external agents.
    #[serde(default)]
    pub mcp: McpConfig,
    /// Monotonic revision bumped on every persisted change. Not serialized.
    #[serde(skip, default)]
    #[ts(skip)]
    pub revision: u64,
}

fn default_capture_regions() -> Vec<detector::CaptureRegion> {
    vec![detector::CaptureRegion::FULL]
}

fn default_sync_enabled() -> bool {
    true
}

fn default_powered() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            lamp_connection: LampConnection::default(),
            lamp_tuning: LampTuning::default(),
            monitor_device_name: None,
            capture_regions: default_capture_regions(),
            sync_enabled: true,
            lamp_mode: LampMode::default(),
            static_color: StaticColor::default(),
            powered: true,
            mcp: McpConfig::default(),
            revision: 0,
        }
    }
}
