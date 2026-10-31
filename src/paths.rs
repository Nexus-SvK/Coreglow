//! Where config, active-profile and status files live.

use std::path::PathBuf;

/// Shipped default profiles, copied to the user's config on first run.
pub const DEFAULT_CONFIG: &str = "/usr/share/coreglow/config.toml";

fn home() -> PathBuf {
    std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("/"))
}

fn xdg_config() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from).unwrap_or_else(|| home().join(".config"))
}

pub fn config_dir() -> PathBuf {
    xdg_config().join("coreglow")
}

/// Where versions before the rename (ccxt-control 0.1) kept their profiles.
fn old_config_dir() -> PathBuf {
    xdg_config().join("ccxt")
}

pub fn config_file() -> PathBuf {
    config_dir().join("config.toml")
}

/// Holds the name of the profile in use; written by `coreglowctl set` and the GUI.
pub fn active_file() -> PathBuf {
    config_dir().join("active")
}

fn runtime_dir() -> PathBuf {
    std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(format!("/run/user/{}", unsafe { libc::getuid() })))
}

/// Live values from the daemon, refreshed every second.
pub fn status_file() -> PathBuf {
    runtime_dir().join("coreglow-status.json")
}

/// Touching this file asks coreglowd to run the fans at full speed for a few seconds.
pub fn test_fans_file() -> PathBuf {
    runtime_dir().join("coreglow-test-fans")
}

/// Ask coreglowd to test the fans (full speed for a few seconds).
pub fn request_fan_test() -> std::io::Result<()> {
    std::fs::write(test_fans_file(), b"")
}

/// Recently used custom colours, newest first (GUI only).
pub fn recent_colors_file() -> PathBuf {
    config_dir().join("recent-colors")
}

/// Create the user's config if it doesn't exist yet: copied from the pre-rename
/// location when there is one, else from the shipped default.
pub fn ensure_config() -> std::io::Result<()> {
    let cfg = config_file();
    if cfg.exists() {
        return Ok(());
    }
    std::fs::create_dir_all(config_dir())?;
    let old = old_config_dir();
    if old.join("config.toml").is_file() {
        std::fs::copy(old.join("config.toml"), &cfg)?;
        if old.join("active").is_file() {
            std::fs::copy(old.join("active"), active_file())?;
        }
        return Ok(());
    }
    let text = std::fs::read_to_string(DEFAULT_CONFIG).unwrap_or_else(|_| crate::config::BUILTIN_DEFAULT.to_string());
    std::fs::write(cfg, text)
}
