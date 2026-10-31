//! ccxtd: drives the Commander Core XT from the active profile.
//!
//! Keeps the controller in software mode like iCUE: streams the lighting and
//! sends fan duties for fans 1 and 2. On exit (or before taking over) it stores a
//! hardware fallback curve, so the fans stay sensible without this daemon.
//! Reconnects by itself when the controller disappears (unplugged, passed to a VM).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use ccxt::config::{Config, Profile};
use ccxt::device::{self, Device};
use ccxt::effects::Temps;
use ccxt::sensors::Sensors;
use ccxt::status::{self, Status};
use ccxt::{paths, zones};

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

struct State {
    profile: Profile,
    name: String,
    sensors: Sensors,
}

fn load() -> Result<State, String> {
    let cfg = Config::load()?;
    let name = cfg.active_name();
    let profile = cfg.profiles[&name].clone();
    let sensors = Sensors::new(&cfg.sensors);
    Ok(State { profile, name, sensors })
}

/// Runs until `stop` is set or the device fails. Err = device error (reconnect).
fn run(dev: &mut Device, st: &mut State, stop: &AtomicBool) -> Result<(), String> {
    dev.wake()?;
    device::store_fallback(dev)?;
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

    while !stop.load(Ordering::Relaxed) {
        let now = Instant::now();
        if now >= next_sensor {
            next_sensor = now + SENSOR_EVERY;
            if stamp() != cfg_stamp {
                cfg_stamp = stamp();
                match load() {
                    Ok(new) => {
                        if new.name != st.name {
                            log(format!("switched to profile {}", new.name));
                        } else {
                            log(format!("profile {} changed", new.name));
                        }
                        *st = new;
                        error = None;
                    }
                    Err(e) => {
                        log(format!("config not loaded, keeping profile {}: {e}", st.name));
                        error = Some(e);
                    }
                }
            }
            let probes = dev.temps()?;
            let mut wanted = st.profile.sensors();
            wanted.extend(["probe1", "probe2", "cpu"].map(String::from));
            wanted.sort();
            wanted.dedup();
            temps = st.sensors.read(&probes, &wanted);
            let fans = st.profile.fan_settings();
            let duties = [fans[0].duty(&temps), fans[1].duty(&temps)];
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
                        .map(|(z, i)| (z.to_string(), ccxt::config::color_hex([last_frame[i * 3], last_frame[i * 3 + 1], last_frame[i * 3 + 2]])))
                        .collect()
                },
                error: error.clone(),
            }
            .write();
        }

        let frame = st.profile.frame((now - t0).as_secs_f64(), &temps);
        if frame != last_frame || now - last_light >= STATIC_EVERY {
            dev.write_frame(&frame)?;
            last_frame = frame;
            last_light = now;
        }
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
        "ccxtd {}: profile {}, {} LEDs ({} connected)",
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
        let _ = dev.wake().and_then(|_| device::store_fallback(&mut dev)).and_then(|_| dev.sleep());
        match result {
            Ok(()) => log("controller back in hardware mode"),
            Err(e) => {
                log(format!("controller error: {e}; reconnecting"));
                std::thread::sleep(RETRY_EVERY);
            }
        }
    }
}
