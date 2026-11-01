//! Live values the daemon publishes for coreglowctl and the GUI.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::config::LayoutConfig;
use crate::model::Model;
use crate::paths;
use crate::zones::Layout;

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
    /// Profile chosen by hand; `profile` differs from it while an automatic rule is in charge.
    #[serde(default)]
    pub manual_profile: String,
    /// The automatic rule in charge, in words.
    #[serde(default)]
    pub auto_rule: Option<String>,
    /// Motherboard sync through OpenRGB: what's connected, or why not (None = off).
    #[serde(default)]
    pub openrgb: Option<String>,
    /// Per RGB port: the layout in use and what the controller detected (read at start).
    #[serde(default)]
    pub devices: Vec<PortInfo>,
    /// Model id of the connected controller ("core-xt", "core").
    #[serde(default)]
    pub controller: Option<String>,
    /// The layout coreglowd built from detection (Commander CORE without a [layout]).
    #[serde(default)]
    pub detected: Option<LayoutConfig>,
    /// Night dimming is on right now.
    #[serde(default)]
    pub night: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PortInfo {
    pub port: String,
    /// Device ids from the layout, in chain order.
    pub configured: Vec<String>,
    /// LEDs the controller detected (ports 1-6 only).
    pub detected_leds: Option<u16>,
    /// Best guess at what that is.
    pub detected_kind: Option<String>,
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

    pub fn model(&self) -> Option<Model> {
        self.controller.as_deref().and_then(Model::from_id)
    }

    /// The detected layout, for the window and CLI (they don't talk to the controller).
    pub fn detected_layout(&self) -> Option<Layout> {
        let d = self.detected.as_ref()?;
        Layout::from_config(self.model()?, &d.ports, &d.names).ok()
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
