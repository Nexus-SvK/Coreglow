//! Temperature sources: the controller's probes, hwmon chips and nvidia-smi.

use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::process::Command;

use crate::effects::Temps;

/// Share of a new reading per second; CPU Tctl jumps a lot, fans shouldn't.
const SMOOTHING: f64 = 0.3;

pub const SENSORS: [(&str, &str); 5] = [
    ("cpu", "Processor (CPU)"),
    ("probe1", "Case probe 1"),
    ("probe2", "Case probe 2"),
    ("igpu", "Built-in graphics"),
    ("gpu", "Graphics card (RTX 3090)"),
];

pub fn label(id: &str) -> &str {
    SENSORS.iter().find(|(i, _)| *i == id).map_or(id, |(_, l)| l)
}

fn default_source(name: &str) -> Option<&'static str> {
    Some(match name {
        "probe1" => "ccxt:0",
        "probe2" => "ccxt:1",
        "cpu" => "hwmon:k10temp:Tctl",
        "igpu" => "hwmon:amdgpu:edge",
        "gpu" => "nvidia-smi",
        _ => return None,
    })
}

pub fn nvidia_available() -> bool {
    std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).any(|d| d.join("nvidia-smi").is_file()))
        .unwrap_or(false)
}

pub struct Sensors {
    sources: BTreeMap<String, String>,
    hwmon: HashMap<(String, String), Option<PathBuf>>,
    smoothed: HashMap<String, f64>,
    nvidia: bool,
}

impl Sensors {
    /// `overrides` comes from the config's [sensors] table.
    pub fn new(overrides: &BTreeMap<String, String>) -> Self {
        Sensors { sources: overrides.clone(), hwmon: HashMap::new(), smoothed: HashMap::new(), nvidia: nvidia_available() }
    }

    fn hwmon(&mut self, chip: &str, label: &str) -> Option<f64> {
        let key = (chip.to_string(), label.to_string());
        let path = self.hwmon.entry(key).or_insert_with(|| find_hwmon(chip, label)).clone()?;
        let raw = std::fs::read_to_string(path).ok()?;
        Some(raw.trim().parse::<f64>().ok()? / 1000.0)
    }

    fn nvidia_smi(&self) -> Option<f64> {
        if !self.nvidia {
            return None;
        }
        let out = Command::new("nvidia-smi")
            .args(["--query-gpu=temperature.gpu", "--format=csv,noheader,nounits"])
            .output()
            .ok()?;
        String::from_utf8_lossy(&out.stdout).split_whitespace().next()?.parse().ok()
    }

    /// Read the named sensors; `probes` are the controller's own readings.
    pub fn read(&mut self, probes: &[Option<f64>], wanted: &[String]) -> Temps {
        let mut out = Temps::new();
        for name in wanted {
            let src = self.sources.get(name).cloned().or_else(|| default_source(name).map(String::from));
            let Some(src) = src else { continue };
            let (kind, rest) = src.split_once(':').unwrap_or((src.as_str(), ""));
            let value = match kind {
                "ccxt" => rest.parse::<usize>().ok().and_then(|i| probes.get(i).copied().flatten()),
                "hwmon" => {
                    let (chip, label) = rest.split_once(':').unwrap_or((rest, ""));
                    self.hwmon(chip, label)
                }
                "nvidia-smi" => self.nvidia_smi(),
                _ => None,
            };
            match value {
                None => {
                    self.smoothed.remove(name);
                }
                Some(v) => {
                    let s = self.smoothed.get(name).map_or(v, |p| p + SMOOTHING * (v - p));
                    self.smoothed.insert(name.clone(), s);
                    out.insert(name.clone(), s);
                }
            }
        }
        out
    }
}

fn find_hwmon(chip: &str, label: &str) -> Option<PathBuf> {
    for dir in std::fs::read_dir("/sys/class/hwmon").ok()?.flatten() {
        let d = dir.path();
        if std::fs::read_to_string(d.join("name")).ok()?.trim() != chip {
            continue;
        }
        for f in std::fs::read_dir(&d).ok()?.flatten() {
            let name = f.file_name().to_string_lossy().to_string();
            if name.starts_with("temp") && name.ends_with("_label")
                && std::fs::read_to_string(f.path()).ok().is_some_and(|l| l.trim() == label)
            {
                return Some(d.join(name.replace("_label", "_input")));
            }
        }
    }
    None
}
