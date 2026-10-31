//! Decides which automatic rule (if any) applies right now.

use std::time::{Duration, Instant};

use crate::config::{parse_hm, Auto};
use crate::effects::Temps;

/// A "temperature above" rule turns off again only this far below its limit (no flapping).
const HYSTERESIS_C: f64 = 3.0;
/// Looking through /proc and asking logind is slower; do it this often.
const SLOW_CHECK_EVERY: Duration = Duration::from_secs(3);

#[derive(Default)]
pub struct AutoState {
    hot: Vec<bool>,
    cmdlines: Vec<String>,
    locked: bool,
    last_slow: Option<Instant>,
}

impl AutoState {
    /// Index of the first matching rule, or None (use the profile chosen by hand).
    pub fn evaluate(&mut self, auto: &Auto, temps: &Temps) -> Option<usize> {
        if !auto.enabled || auto.rules.is_empty() {
            return None;
        }
        if self.last_slow.is_none_or(|t| t.elapsed() >= SLOW_CHECK_EVERY) {
            self.last_slow = Some(Instant::now());
            if auto.rules.iter().any(|r| r.when == "process") {
                self.cmdlines = cmdlines();
            }
            if auto.rules.iter().any(|r| r.when == "locked") {
                self.locked = screen_locked();
            }
        }
        self.hot.resize(auto.rules.len(), false);
        let now = local_minutes();
        let mut first = None;
        for (i, r) in auto.rules.iter().enumerate() {
            let matched = match r.when.as_str() {
                "temp-above" => {
                    let limit = r.temp.map_or(f64::MAX, |t| t.0);
                    let t = temps.get(r.sensor.as_deref().unwrap_or("cpu")).copied();
                    let on = match t {
                        Some(t) if self.hot[i] => t > limit - HYSTERESIS_C,
                        Some(t) => t >= limit,
                        None => false,
                    };
                    self.hot[i] = on;
                    on
                }
                "process" => {
                    let needle = r.name.as_deref().unwrap_or("").to_lowercase();
                    !needle.is_empty() && self.cmdlines.iter().any(|c| c.contains(&needle))
                }
                "time" => match (r.from.as_deref().and_then(parse_hm), r.to.as_deref().and_then(parse_hm)) {
                    (Some(a), Some(b)) if a <= b => now >= a && now < b,
                    (Some(a), Some(b)) => now >= a || now < b, // across midnight
                    _ => false,
                },
                "locked" => self.locked,
                _ => false,
            };
            if matched && first.is_none() {
                first = Some(i);
            }
        }
        first
    }
}

/// Minutes after local midnight.
pub fn local_minutes() -> u32 {
    unsafe {
        let t = libc::time(std::ptr::null_mut());
        let mut tm: libc::tm = std::mem::zeroed();
        libc::localtime_r(&t, &mut tm);
        (tm.tm_hour * 60 + tm.tm_min) as u32
    }
}

/// Lower-cased command lines of this user's processes.
fn cmdlines() -> Vec<String> {
    let uid = unsafe { libc::getuid() };
    let mut out = Vec::new();
    let Ok(dir) = std::fs::read_dir("/proc") else { return out };
    for e in dir.flatten() {
        let name = e.file_name();
        if !name.to_string_lossy().bytes().all(|b| b.is_ascii_digit()) {
            continue;
        }
        let path = e.path();
        let mine = std::fs::read_to_string(path.join("status"))
            .ok()
            .and_then(|s| s.lines().find(|l| l.starts_with("Uid:")).map(String::from))
            .and_then(|l| l.split_whitespace().nth(1).and_then(|u| u.parse::<u32>().ok()))
            == Some(uid);
        if !mine {
            continue;
        }
        if let Ok(raw) = std::fs::read(path.join("cmdline")) {
            let s = String::from_utf8_lossy(&raw).replace('\0', " ").to_lowercase();
            if !s.trim().is_empty() {
                out.push(s);
            }
        }
    }
    out
}

/// Is this user's graphical session locked? (logind's LockedHint)
fn screen_locked() -> bool {
    let run = |args: &[&str]| {
        std::process::Command::new("loginctl")
            .args(args)
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .filter(|s| !s.is_empty())
    };
    let uid = unsafe { libc::getuid() }.to_string();
    let Some(session) = run(&["show-user", &uid, "--property=Display", "--value"]) else { return false };
    run(&["show-session", &session, "--property=LockedHint", "--value"]).as_deref() == Some("yes")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Num, Rule};

    #[test]
    fn first_matching_rule_wins_with_hysteresis() {
        let auto = Auto {
            enabled: true,
            rules: vec![
                Rule { temp: Some(Num(80.0)), ..Rule::new("temp-above", "hot") },
                Rule { from: Some("00:00".into()), to: Some("23:59".into()), ..Rule::new("time", "always") },
            ],
        };
        let mut st = AutoState::default();
        let temps = |t: f64| -> Temps { [("cpu".to_string(), t)].into() };
        assert_eq!(st.evaluate(&auto, &temps(70.0)), Some(1));
        assert_eq!(st.evaluate(&auto, &temps(81.0)), Some(0));
        assert_eq!(st.evaluate(&auto, &temps(78.0)), Some(0), "stays on within the hysteresis");
        assert_eq!(st.evaluate(&auto, &temps(76.0)), Some(1));
        let off = Auto { enabled: false, ..auto };
        assert_eq!(st.evaluate(&off, &temps(90.0)), None);
    }
}
