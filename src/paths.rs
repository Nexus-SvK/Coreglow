//! Where config, active-profile and status files live.

use std::path::PathBuf;

/// Shipped default profiles, copied to the user's config on first run.
pub const DEFAULT_CONFIG: &str = "/usr/share/ccxt-control/config.toml";

fn home() -> PathBuf {
    std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("/"))
}

pub fn config_dir() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".config"))
        .join("ccxt")
}

pub fn config_file() -> PathBuf {
    config_dir().join("config.toml")
}

/// Holds the name of the profile in use; written by `ccxtctl set` and the GUI.
pub fn active_file() -> PathBuf {
    config_dir().join("active")
}

/// Live values from the daemon, refreshed every second.
pub fn status_file() -> PathBuf {
    let dir = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(format!("/run/user/{}", unsafe { libc::getuid() })));
    dir.join("ccxt-status.json")
}

/// Create the user's config from the shipped default if it doesn't exist yet.
pub fn ensure_config() -> std::io::Result<()> {
    let cfg = config_file();
    if cfg.exists() {
        return Ok(());
    }
    std::fs::create_dir_all(config_dir())?;
    let text = std::fs::read_to_string(DEFAULT_CONFIG).unwrap_or_else(|_| crate::config::BUILTIN_DEFAULT.to_string());
    std::fs::write(cfg, text)
}
