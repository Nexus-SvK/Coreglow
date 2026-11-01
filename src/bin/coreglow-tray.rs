//! coreglow-tray: system-tray icon for quick changes (StatusNotifierItem, works on KDE
//! and on GNOME with the AppIndicator extension).
//!
//! Profile switching, automatic switching on/off, test fans, open the window. Talks to
//! coreglowd through its control socket (live status), falling back to the files.

use std::time::Duration;

use ksni::blocking::TrayMethods;
use ksni::menu::{CheckmarkItem, RadioGroup, RadioItem, StandardItem};
use ksni::MenuItem;

use coreglow::config::Config;
use coreglow::control;
use coreglow::i18n::{tr, trf};
use coreglow::status::Status;

#[derive(Default)]
struct Tray {
    profiles: Vec<String>,
    manual: String,
    auto_enabled: bool,
    status: Option<Status>,
}

impl Tray {
    fn refresh(&mut self) {
        if let Ok(cfg) = Config::load() {
            self.manual = cfg.active_name();
            self.profiles = cfg.profiles.keys().cloned().collect();
            self.auto_enabled = cfg.auto.enabled;
        }
        self.status = self.status.take().or_else(Status::read).filter(|s| s.fresh());
    }

    fn summary(&self) -> String {
        match &self.status {
            None => tr("Lighting service isn't running").into(),
            Some(s) => {
                let cpu = s.temps.get("cpu").map_or(String::new(), |t| trf("CPU {} °C · ", &[&format!("{t:.0}")]));
                let rpm = |k: &str| s.rpm.get(k).copied().unwrap_or(0);
                format!("{cpu}{}", trf("fans {}/{} rpm · pump {} rpm", &[&rpm("fan1"), &rpm("fan2"), &rpm("pump")]))
            }
        }
    }
}

fn open_window() {
    let _ = std::process::Command::new("coreglow").spawn();
}

impl ksni::Tray for Tray {
    fn id(&self) -> String {
        "coreglow".into()
    }

    fn title(&self) -> String {
        "Coreglow".into()
    }

    fn icon_name(&self) -> String {
        "coreglow".into()
    }

    fn tool_tip(&self) -> ksni::ToolTip {
        let profile = self.status.as_ref().map_or(self.manual.clone(), |s| s.profile.clone());
        ksni::ToolTip {
            title: format!("Coreglow — {profile}"),
            description: self.summary(),
            ..Default::default()
        }
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        open_window();
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let mut items: Vec<MenuItem<Self>> = vec![StandardItem { label: self.summary(), enabled: false, ..Default::default() }.into()];
        if let Some(rule) = self.status.as_ref().and_then(|s| s.auto_rule.clone()) {
            let profile = self.status.as_ref().map_or(String::new(), |s| s.profile.clone());
            items.push(StandardItem { label: trf("Automatic: “{}” because {}", &[&profile, &rule]), enabled: false, ..Default::default() }.into());
        }
        items.push(MenuItem::Separator);
        let selected = self.profiles.iter().position(|p| *p == self.manual).unwrap_or(0);
        items.push(
            RadioGroup {
                selected,
                select: Box::new(|this: &mut Self, i| {
                    if let Some(name) = this.profiles.get(i).cloned() {
                        if control::set_profile_or_file(&name).is_ok() {
                            this.manual = name;
                        }
                    }
                }),
                options: self.profiles.iter().map(|p| RadioItem { label: p.clone(), ..Default::default() }).collect(),
            }
            .into(),
        );
        items.push(MenuItem::Separator);
        items.push(
            CheckmarkItem {
                label: tr("Switch profiles automatically").into(),
                checked: self.auto_enabled,
                activate: Box::new(|this: &mut Self| {
                    if let Ok(mut cfg) = Config::load() {
                        cfg.auto.enabled = !cfg.auto.enabled;
                        cfg.active = cfg.active_name();
                        if cfg.save().is_ok() {
                            this.auto_enabled = cfg.auto.enabled;
                        }
                    }
                }),
                ..Default::default()
            }
            .into(),
        );
        items.push(
            StandardItem {
                label: tr("Test fans (full speed for 5 s)").into(),
                enabled: self.status.is_some(),
                activate: Box::new(|_: &mut Self| {
                    let _ = control::test_fans();
                }),
                ..Default::default()
            }
            .into(),
        );
        items.push(MenuItem::Separator);
        items.push(StandardItem { label: tr("Open Coreglow").into(), icon_name: "coreglow".into(), activate: Box::new(|_: &mut Self| open_window()), ..Default::default() }.into());
        items.push(StandardItem { label: tr("Hide this icon").into(), activate: Box::new(|_: &mut Self| std::process::exit(0)), ..Default::default() }.into());
        items
    }
}

fn main() {
    let mut tray = Tray::default();
    tray.refresh();
    let handle = match tray.spawn() {
        Ok(h) => h,
        Err(e) => {
            eprintln!("coreglow-tray: no system tray available: {e}");
            std::process::exit(1);
        }
    };
    // live status from coreglowd (pushed through the control socket)
    let live = handle.clone();
    std::thread::spawn(move || {
        control::watch(|st| {
            if live.update(|t: &mut Tray| t.status = Some(st).filter(Status::fresh)).is_none() {
                std::process::exit(0);
            }
        })
    });
    loop {
        std::thread::sleep(Duration::from_secs(2));
        if handle.update(|t: &mut Tray| t.refresh()).is_none() {
            break; // tray service ended
        }
    }
}
