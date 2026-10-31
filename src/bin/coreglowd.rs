//! coreglowd: drives the Commander Core XT from the active profile.
//!
//! Keeps the controller in software mode like iCUE: streams the lighting and
//! sends fan duties for fans 1 and 2. On exit (or before taking over) it stores a
//! hardware fallback curve, so the fans stay sensible without this daemon.
//! Reconnects by itself when the controller disappears (unplugged, passed to a VM).

use std::collections::{BTreeMap, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use coreglow::auto::AutoState;
use coreglow::config::{Config, OpenRgb, Profile};
use coreglow::device::{self, Device};
use coreglow::effects::Temps;
use coreglow::sensors::Sensors;
use coreglow::status::{self, Status};
use coreglow::{openrgb, paths, zones};

const FPS: u64 = 30;
const SENSOR_EVERY: Duration = Duration::from_secs(1);
const RESEND_EVERY: Duration = Duration::from_secs(10);
/// Unchanged frames are still re-sent this often (keeps software mode alive).
const STATIC_EVERY: Duration = Duration::from_secs(2);
const RETRY_EVERY: Duration = Duration::from_secs(3);

fn log(msg: impl AsRef<str>) {
    eprintln!("{}", msg.as_ref());
}

fn stamp() -> (Option<SystemTime>, Option<SystemTime>) {
    let m = |p: std::path::PathBuf| std::fs::metadata(p).and_then(|m| m.modified()).ok();
    (m(paths::config_file()), m(paths::active_file()))
}

/// How long a "test fans" request runs them at full speed.
const TEST_FANS_FOR: Duration = Duration::from_secs(5);

struct State {
    cfg: Config,
    /// Profile chosen by hand (GUI, CLI, tray).
    manual: String,
    /// Profile in use: `manual`, or the one an automatic rule picked.
    name: String,
    profile: Profile,
    /// Index of the automatic rule in charge, if any.
    rule: Option<usize>,
    sensors: Sensors,
    fallback: Vec<(f64, f64)>,
}

fn load() -> Result<State, String> {
    let cfg = Config::load()?;
    let name = cfg.active_name();
    let profile = cfg.profiles[&name].clone();
    let sensors = Sensors::new(&cfg.sensors);
    let fallback = cfg.fallback_curve();
    Ok(State { manual: name.clone(), name, profile, rule: None, sensors, fallback, cfg })
}

impl State {
    /// Use the profile rule `rule` picks (or the manual one), logging the change.
    fn apply_rule(&mut self, rule: Option<usize>) {
        let target = match rule {
            Some(i) => self.cfg.auto.rules[i].profile.clone(),
            None => self.manual.clone(),
        };
        if target != self.name || rule != self.rule {
            match rule {
                Some(i) => log(format!("automatic: {} → profile {target}", self.cfg.auto.rules[i].describe())),
                None if self.rule.is_some() => log(format!("automatic rules no longer match → profile {target}")),
                None => {}
            }
        }
        self.rule = rule;
        if target != self.name {
            self.profile = self.cfg.profiles[&target].clone();
            self.name = target;
        }
    }
}

/// Sends the "motherboard" part of the profile to an OpenRGB server, when enabled.
struct MotherboardSync {
    client: Option<openrgb::Client>,
    next_try: Instant,
    last: Vec<[u8; 3]>,
    last_sent: Instant,
    /// For the status file: what's connected, or why not.
    state: Option<String>,
}

impl MotherboardSync {
    const RETRY: Duration = Duration::from_secs(10);

    fn new() -> Self {
        MotherboardSync { client: None, next_try: Instant::now(), last: Vec::new(), last_sent: Instant::now(), state: None }
    }

    fn tick(&mut self, cfg: &OpenRgb, profile: &Profile, t: f64, temps: &Temps) {
        if !cfg.enabled {
            if self.client.take().is_some() {
                log("motherboard sync off");
            }
            self.state = None;
            return;
        }
        let now = Instant::now();
        if self.client.is_none() {
            if now < self.next_try {
                return;
            }
            self.next_try = now + Self::RETRY;
            match openrgb::Client::connect(cfg.host(), cfg.port()) {
                Ok(mut c) => {
                    let names: Vec<String> = c.controllers.iter().map(|d| format!("{} ({} LEDs)", d.name, d.leds)).collect();
                    for i in 0..c.controllers.len() as u32 {
                        let _ = c.set_custom_mode(i);
                    }
                    log(format!("motherboard sync: {}", names.join(", ")));
                    self.state = Some(names.join(", "));
                    self.last.clear();
                    self.client = Some(c);
                }
                Err(e) => {
                    if self.state.as_deref() != Some(e.as_str()) {
                        log(format!("motherboard sync: {e}"));
                    }
                    self.state = Some(e);
                    return;
                }
            }
        }
        let Some(c) = self.client.as_mut() else { return };
        let colors = profile.motherboard(t, temps, c.total_leds());
        if colors == self.last && now - self.last_sent < STATIC_EVERY {
            return;
        }
        let mut start = 0;
        let mut result = Ok(());
        for d in c.controllers.clone() {
            result = result.and_then(|_| c.set_leds(d.index, &colors[start..start + d.leds]));
            start += d.leds;
        }
        match result {
            Ok(()) => {
                self.last = colors;
                self.last_sent = now;
            }
            Err(e) => {
                log(format!("motherboard sync lost: {e}"));
                self.state = Some(e);
                self.client = None;
            }
        }
    }

    fn first_color(&self) -> Option<[u8; 3]> {
        self.client.as_ref().and(self.last.first().copied())
    }
}

/// True while a "test fans" request (the request file touched by the GUI or CLI) is recent.
fn testing_fans() -> bool {
    std::fs::metadata(paths::test_fans_file())
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.elapsed().ok())
        .is_some_and(|age| age < TEST_FANS_FOR)
}

/// Runs until `stop` is set or the device fails. Err = device error (reconnect).
fn run(dev: &mut Device, st: &mut State, stop: &AtomicBool) -> Result<(), String> {
    dev.wake()?;
    device::store_fallback(dev, &st.fallback)?;
    dev.init_lighting()?;
    let all_parts = zones::PICTURE_ZONES.map(|z| (z, zones::zone(z)[0]));
    let t0 = Instant::now();
    let mut cfg_stamp = stamp();
    let mut temps = Temps::new();
    let mut sent: Option<[u8; 2]> = None;
    let (mut last_fan, mut last_light) = (Instant::now() - RESEND_EVERY, Instant::now() - STATIC_EVERY);
    let mut next_sensor = Instant::now();
    let mut last_frame: Vec<u8> = Vec::new();
    let mut error = None;
    let mut history: BTreeMap<String, VecDeque<f64>> = BTreeMap::new();
    let mut next_history = Instant::now();
    let mut auto = AutoState::default();
    let mut board = MotherboardSync::new();

    while !stop.load(Ordering::Relaxed) {
        let now = Instant::now();
        // Checked every frame (two stat calls), so edits and profile switches show at once.
        if stamp() != cfg_stamp {
            cfg_stamp = stamp();
            match load() {
                Ok(mut new) => {
                    if new.manual != st.manual {
                        log(format!("switched to profile {}", new.manual));
                    } else {
                        log(format!("profile {} changed", new.manual));
                    }
                    if new.fallback != st.fallback {
                        device::store_fallback(dev, &new.fallback)?;
                        log("backup curve stored in the controller");
                    }
                    // keep the rule in charge until the next check, if it still exists
                    let rule = st.rule.filter(|i| *i < new.cfg.auto.rules.len() && new.cfg.auto.enabled);
                    new.rule = st.rule;
                    new.apply_rule(rule);
                    *st = new;
                    error = None;
                }
                Err(e) => {
                    log(format!("config not loaded, keeping profile {}: {e}", st.name));
                    error = Some(e);
                }
            }
        }
        if now >= next_sensor {
            next_sensor = now + SENSOR_EVERY;
            let probes = dev.temps()?;
            let mut wanted = st.profile.sensors();
            for p in st.cfg.profiles.values() {
                wanted.extend(p.sensors());
            }
            wanted.extend(st.cfg.auto.rules.iter().filter_map(|r| r.sensor.clone()));
            wanted.extend(["probe1", "probe2", "cpu"].map(String::from));
            wanted.sort();
            wanted.dedup();
            temps = st.sensors.read(&probes, &wanted);
            let rule = auto.evaluate(&st.cfg.auto, &temps);
            st.apply_rule(rule);
            let fans = st.profile.fan_settings();
            let testing = testing_fans();
            let duties = if testing { [100, 100] } else { [fans[0].duty(&temps), fans[1].duty(&temps)] };
            let moved = sent.is_none_or(|s| s.iter().zip(duties).any(|(a, b)| a.abs_diff(b) >= 2));
            if moved || now - last_fan >= RESEND_EVERY {
                dev.set_duties(&[(0, duties[0]), (1, duties[1])])?;
                if sent != Some(duties) {
                    log(format!("fans {}/{} %", duties[0], duties[1]));
                }
                sent = Some(duties);
                last_fan = now;
            }
            let rpm = dev.speeds()?;
            if now >= next_history {
                next_history = now + Duration::from_secs(status::HISTORY_EVERY_S);
                let r0 = rpm.first().copied().unwrap_or(0) as f64;
                let samples = [("cpu", temps.get("cpu").copied()), ("probe1", temps.get("probe1").copied()), ("fan1_rpm", Some(r0))];
                for (key, value) in samples {
                    if let Some(v) = value {
                        let h = history.entry(key.to_string()).or_default();
                        h.push_back((v * 10.0).round() / 10.0);
                        while h.len() > status::HISTORY_LEN {
                            h.pop_front();
                        }
                    }
                }
            }
            let r = |i: usize| rpm.get(i).copied().unwrap_or(0);
            Status {
                profile: st.name.clone(),
                time: status::now(),
                temps: temps.iter().map(|(k, v)| (k.clone(), (v * 10.0).round() / 10.0)).collect(),
                fan_duty: [("fan1".into(), duties[0]), ("fan2".into(), duties[1])].into(),
                rpm: [("fan1".into(), r(0)), ("fan2".into(), r(1)), ("pump".into(), r(device::PUMP_CHANNEL))].into(),
                zones: if last_frame.is_empty() {
                    Default::default()
                } else {
                    all_parts
                        .iter()
                        .map(|(z, i)| (z.to_string(), coreglow::config::color_hex([last_frame[i * 3], last_frame[i * 3 + 1], last_frame[i * 3 + 2]])))
                        .chain(board.first_color().map(|c| ("motherboard".to_string(), coreglow::config::color_hex(c))))
                        .collect()
                },
                openrgb: board.state.clone(),
                error: error.clone(),
                history: history.iter().map(|(k, v)| (k.clone(), v.iter().copied().collect())).collect(),
                testing_fans: testing,
                manual_profile: st.manual.clone(),
                auto_rule: st.rule.map(|i| st.cfg.auto.rules[i].describe()),
            }
            .write();
        }

        let frame = st.profile.frame((now - t0).as_secs_f64(), &temps);
        if frame != last_frame || now - last_light >= STATIC_EVERY {
            dev.write_frame(&frame)?;
            last_frame = frame;
            last_light = now;
        }
        board.tick(&st.cfg.openrgb, &st.profile, (now - t0).as_secs_f64(), &temps);
        let pause = if st.profile.animated() { Duration::from_millis(1000 / FPS) } else { Duration::from_millis(200) };
        std::thread::sleep(pause);
    }
    Ok(())
}

fn main() {
    let stop = Arc::new(AtomicBool::new(false));
    {
        let stop = stop.clone();
        ctrlc::set_handler(move || stop.store(true, Ordering::Relaxed)).expect("signal handler");
    }
    let mut st = match load() {
        Ok(s) => s,
        Err(e) => {
            log(format!("can't start: {e}"));
            std::process::exit(1);
        }
    };
    log(format!(
        "coreglowd {}: profile {}, {} LEDs ({} connected)",
        env!("CARGO_PKG_VERSION"),
        st.name,
        zones::frame_len(),
        zones::zone("all").len()
    ));

    let mut waiting_logged = false;
    while !stop.load(Ordering::Relaxed) {
        let mut dev = match Device::open() {
            Ok(d) => d,
            Err(e) => {
                if !waiting_logged {
                    log(format!("waiting for the controller: {e}"));
                    waiting_logged = true;
                }
                Status { profile: st.name.clone(), time: status::now(), error: Some(e), ..Default::default() }.write();
                std::thread::sleep(RETRY_EVERY);
                continue;
            }
        };
        waiting_logged = false;
        log(format!("controller at {}", dev.path.display()));
        let result = run(&mut dev, &mut st, &stop);
        // Hand the controller back to its own fallback curve, whatever happened.
        let _ = dev.wake().and_then(|_| device::store_fallback(&mut dev, &st.fallback)).and_then(|_| dev.sleep());
        match result {
            Ok(()) => log("controller back in hardware mode"),
            Err(e) => {
                log(format!("controller error: {e}; reconnecting"));
                std::thread::sleep(RETRY_EVERY);
            }
        }
    }
}
