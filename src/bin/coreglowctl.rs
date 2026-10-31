//! coreglowctl: list and switch profiles, show live values, restore the fallback curve.

use coreglow::config::{Config, Num};
use coreglow::device::{self, Device};
use coreglow::status::Status;
use coreglow::{paths, sensors, zones};

const USAGE: &str = "\
usage: coreglowctl COMMAND

  status            live temperatures, fan speeds and the profile in use (default)
  list              all profiles (* = in use)
  set NAME          switch to profile NAME
  brightness NAME N set the brightness of profile NAME's main lighting to N %
  edit              open the profiles file in $EDITOR
  path              print where the profiles file is
  test-fans         run the radiator fans at full speed for 5 seconds
  auto [on|off]     show the automatic switching rules, or turn them on or off
  motherboard [on|off]  send the lighting to the motherboard too (through the OpenRGB server)
  devices           what's connected to each RGB port: the layout in use and what the controller detects
  fallback          store the hardware fallback fan curve (only while coreglowd is stopped)
  gui               open the window (coreglow)";

fn fail(msg: impl std::fmt::Display) -> ! {
    eprintln!("coreglowctl: {msg}");
    std::process::exit(1)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = args.first().map(String::as_str).unwrap_or("status");
    match (cmd, args.len()) {
        ("status", 1) | ("status", 0) => status(),
        ("list", 1) => {
            let cfg = Config::load().unwrap_or_else(|e| fail(e));
            let active = cfg.active_name();
            for (name, p) in &cfg.profiles {
                let fans = &p.fans;
                let fan = match fans.fixed {
                    Some(f) => format!("fixed {} %", f.0),
                    None => format!("follows {}", sensors::label(&fans.sensor_name())),
                };
                let light = serde_json::to_value(p.lighting.effect).ok().and_then(|v| v.as_str().map(String::from)).unwrap_or_default();
                println!("{} {name:16} lighting: {light:13} fans: {fan}", if *name == active { '*' } else { ' ' });
            }
        }
        ("set", 2) => {
            let cfg = Config::load().unwrap_or_else(|e| fail(e));
            let name = &args[1];
            if !cfg.profiles.contains_key(name) {
                fail(format!("no profile '{name}'; try: {}", cfg.profiles.keys().cloned().collect::<Vec<_>>().join(", ")));
            }
            std::fs::write(paths::active_file(), format!("{name}\n")).unwrap_or_else(|e| fail(e));
            println!("profile {name} (applies within a second)");
        }
        ("brightness", 3) => {
            let mut cfg = Config::load().unwrap_or_else(|e| fail(e));
            cfg.active = cfg.active_name();
            let n: f64 = args[2].trim_end_matches('%').parse().unwrap_or_else(|_| fail("brightness must be 0-100"));
            let p = cfg.profiles.get_mut(&args[1]).unwrap_or_else(|| fail(format!("no profile '{}'", args[1])));
            p.lighting.brightness = Some(Num(n.clamp(0.0, 100.0)));
            cfg.save().unwrap_or_else(|e| fail(e));
            println!("{} brightness {} %", args[1], n.clamp(0.0, 100.0));
        }
        ("edit", 1) => {
            paths::ensure_config().unwrap_or_else(|e| fail(e));
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".into());
            let ok = std::process::Command::new(&editor).arg(paths::config_file()).status();
            if let Err(e) = ok {
                fail(format!("{editor}: {e}"));
            }
            if let Err(e) = Config::load() {
                fail(format!("the file has a problem, the daemon keeps the previous settings: {e}"));
            }
        }
        ("path", 1) => println!("{}", paths::config_file().display()),
        ("fallback", 1) => {
            if Status::read().is_some_and(|s| s.fresh()) {
                fail("coreglowd is running; stop it first: systemctl --user stop coreglowd");
            }
            let curve = Config::load().unwrap_or_else(|e| fail(e)).fallback_curve();
            let mut dev = Device::open().unwrap_or_else(|e| fail(e));
            dev.wake()
                .and_then(|_| device::store_fallback(&mut dev, &curve))
                .and_then(|_| dev.sleep())
                .unwrap_or_else(|e| fail(e));
            let pts: Vec<String> = curve.iter().map(|(t, d)| format!("{t}°C {d}%")).collect();
            println!("fallback stored: fans 1-2 follow probe 1 ({}), pump 100 %", pts.join(", "));
        }
        ("devices", 1) => {
            let cfg = Config::load().unwrap_or_else(|e| fail(e));
            let layout = cfg.layout().unwrap_or_else(|e| fail(e));
            println!("layout: {}", if cfg.layout.is_some() { "from [layout] in the profiles file" } else { "this build's (built in)" });
            let status = Status::read().filter(|s| s.fresh());
            for (i, port) in zones::PORTS.iter().enumerate() {
                let devs: Vec<&str> = layout.ports[i].iter().map(|k| k.id).collect();
                let leds: usize = layout.ports[i].iter().map(|k| k.leds).sum();
                let configured = if devs.is_empty() { "nothing".to_string() } else { format!("{} ({leds} LEDs)", devs.join(", ")) };
                let detected = status
                    .as_ref()
                    .and_then(|s| s.devices.iter().find(|d| d.port == *port))
                    .map(|d| match (d.detected_leds, &d.detected_kind) {
                        (Some(n), Some(k)) => format!("detected {n} LEDs: {k}"),
                        (Some(n), None) => format!("detected {n} LEDs"),
                        (None, _) if i == 0 => "not auto-detected".to_string(),
                        (None, _) => "nothing detected".to_string(),
                    })
                    .unwrap_or_else(|| "(start coreglowd to see detection)".into());
                println!("  {port:9} {configured:40} {detected}");
            }
            println!("parts: {}", layout.zone_names().join(", "));
        }
        ("motherboard", 1) | ("motherboard", 2) => {
            let mut cfg = Config::load().unwrap_or_else(|e| fail(e));
            if let Some(arg) = args.get(1) {
                cfg.openrgb.enabled = match arg.as_str() {
                    "on" => true,
                    "off" => false,
                    _ => fail("use: coreglowctl motherboard on|off"),
                };
                cfg.active = cfg.active_name();
                cfg.save().unwrap_or_else(|e| fail(e));
            }
            println!(
                "motherboard sync is {} (OpenRGB server {}:{})",
                if cfg.openrgb.enabled { "on" } else { "off" },
                cfg.openrgb.host(),
                cfg.openrgb.port()
            );
            if let Some(state) = Status::read().filter(|s| s.fresh()).and_then(|s| s.openrgb) {
                println!("  {state}");
            }
        }
        ("auto", 1) | ("auto", 2) => {
            let mut cfg = Config::load().unwrap_or_else(|e| fail(e));
            if let Some(arg) = args.get(1) {
                cfg.auto.enabled = match arg.as_str() {
                    "on" => true,
                    "off" => false,
                    _ => fail("use: coreglowctl auto on|off"),
                };
                cfg.active = cfg.active_name();
                cfg.save().unwrap_or_else(|e| fail(e));
            }
            println!("automatic switching is {}", if cfg.auto.enabled { "on" } else { "off" });
            if cfg.auto.rules.is_empty() {
                println!("  no rules yet: add them on the Profiles page of the window, or as [auto] in {}", paths::config_file().display());
            }
            for (i, r) in cfg.auto.rules.iter().enumerate() {
                println!("  {}. when {} → {}", i + 1, r.describe(), r.profile);
            }
        }
        ("test-fans", 1) => {
            if !Status::read().is_some_and(|s| s.fresh()) {
                fail("coreglowd isn't running; start it first: systemctl --user start coreglowd");
            }
            paths::request_fan_test().unwrap_or_else(|e| fail(e));
            println!("fans at full speed for 5 seconds");
        }
        ("gui", 1) => {
            let err = std::process::Command::new("coreglow").spawn();
            if let Err(e) = err {
                fail(format!("coreglow: {e}"));
            }
        }
        _ => {
            eprintln!("{USAGE}");
            std::process::exit(2);
        }
    }
}

fn status() {
    let Some(s) = Status::read() else { fail("no status: is coreglowd running? (systemctl --user status coreglowd)") };
    if !s.fresh() {
        println!("coreglowd isn't running (last status is old): systemctl --user start coreglowd");
    }
    if let Some(e) = &s.error {
        println!("problem  {e}");
    }
    println!("profile  {}", s.profile);
    if let Some(rule) = &s.auto_rule {
        println!("auto     in use because {rule} (chosen by hand: {})", s.manual_profile);
    }
    println!("temps    {}", s.temps.iter().map(|(k, v)| format!("{} {v:.1} °C", sensors::label(k))).collect::<Vec<_>>().join("   "));
    println!("fans     {}", s.fan_duty.iter().map(|(k, v)| format!("{k} {v} %")).collect::<Vec<_>>().join("   "));
    println!("rpm      {}", s.rpm.iter().map(|(k, v)| format!("{k} {v}")).collect::<Vec<_>>().join("   "));
}
