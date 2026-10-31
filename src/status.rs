//! Live values the daemon publishes for coreglowctl and the GUI.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::paths;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Status {
    pub profile: String,
    /// Unix time of this snapshot.
    pub time: f64,
    pub temps: BTreeMap<String, f64>,
    pub fan_duty: BTreeMap<String, u8>,
    pub rpm: BTreeMap<String, u16>,
    /// Current colour of each part ("RRGGBB"), for the GUI's picture.
    pub zones: BTreeMap<String, String>,
    #[serde(default)]
    pub error: Option<String>,
    /// Recent readings, oldest first (one every HISTORY_EVERY_S seconds), for the GUI's graphs.
    #[serde(default)]
    pub history: BTreeMap<String, Vec<f64>>,
    /// Fans are running at full speed for a "test fans" request.
    #[serde(default)]
    pub testing_fans: bool,
}

pub const HISTORY_EVERY_S: u64 = 10;
/// 60 samples x 10 s = the last 10 minutes.
pub const HISTORY_LEN: usize = 60;

pub fn now() -> f64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0.0, |d| d.as_secs_f64())
}

impl Status {
    pub fn read() -> Option<Status> {
        serde_json::from_str(&std::fs::read_to_string(paths::status_file()).ok()?).ok()
    }

    /// The daemon writes every second; older than a few seconds means it isn't running.
    pub fn fresh(&self) -> bool {
        now() - self.time < 6.0
    }

    pub fn write(&self) {
        let path = paths::status_file();
        let tmp = path.with_extension("tmp");
        if let Ok(text) = serde_json::to_string(self) {
            if std::fs::write(&tmp, text).is_ok() {
                let _ = std::fs::rename(tmp, path);
            }
        }
    }
}
