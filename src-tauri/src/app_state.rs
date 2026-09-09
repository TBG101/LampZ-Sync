use std::sync::{Arc, Condvar, Mutex, RwLock};

use crate::{color::Rgb, config::{Config, LampConnection, LampTuning}};

pub type SharedMailbox = Arc<(Mutex<LampMailbox>, Condvar)>;

pub struct LampMailbox {
    pub latest_color: Option<Rgb>,
    pub connection_changed: Option<LampConnection>,
    pub lamp_tuning_changed: Option<LampTuning>,
}


pub struct AppState {
    pub config: RwLock<Config>,
    pub mailbox: SharedMailbox,
    pub monitor_sender: std::sync::mpsc::Sender<String>,
}