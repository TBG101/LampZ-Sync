use std::sync::{Arc, Condvar, Mutex, RwLock};

use crate::{color::Rgb, config::Config};

pub type SharedMailbox = Arc<(Mutex<LampMailbox>, Condvar)>;

/// Cross-thread signal bus.
///
/// The mailbox deliberately holds **no config data**; `AppState.config` is the
/// single source of truth. It only carries the high-frequency color stream and
/// an epoch that wakes workers when the persisted config changes.
#[derive(Default)]
pub struct LampMailbox {
    pub latest_color: Option<Rgb>,
    pub config_epoch: u64,
}

pub struct AppState {
    pub config: RwLock<Config>,
    pub mailbox: SharedMailbox,
    pub monitor_sender: std::sync::mpsc::Sender<String>,
}
