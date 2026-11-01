//! Control socket ($XDG_RUNTIME_DIR/coreglowd.sock): one text line per request.
//!
//!   set NAME     use profile NAME (also written to the `active` file, so it sticks)
//!   test-fans    fans at full speed for a few seconds, starting now
//!   status       the latest status as one JSON line
//!   watch        a JSON status line every time coreglowd publishes one (until the client hangs up)
//!
//! Replies are one line: "ok", "error: …", or JSON. The files stay the source of truth:
//! when coreglowd isn't running (or is older), clients fall back to them.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use crate::paths;
use crate::status::Status;

pub fn socket_path() -> PathBuf {
    paths::runtime_dir().join("coreglowd.sock")
}

/// What the main loop is asked to do.
#[derive(Debug, PartialEq)]
pub enum Request {
    TestFans,
}

/// Latest status line plus a counter, so watchers wake on each new one.
#[derive(Default)]
pub struct Hub {
    latest: Mutex<(u64, String)>,
    changed: Condvar,
}

impl Hub {
    pub fn publish(&self, status: &Status) {
        let Ok(line) = serde_json::to_string(status) else { return };
        let mut l = self.latest.lock().unwrap();
        *l = (l.0 + 1, line);
        self.changed.notify_all();
    }

    fn latest(&self) -> String {
        self.latest.lock().unwrap().1.clone()
    }

    /// Wait for a status newer than `seen` (or time out); returns (counter, line).
    fn next(&self, seen: u64) -> (u64, String) {
        let guard = self.latest.lock().unwrap();
        let (guard, _) = self.changed.wait_timeout_while(guard, Duration::from_secs(5), |l| l.0 == seen).unwrap();
        guard.clone()
    }
}

/// Start listening (coreglowd). Requests that need the main loop go to `tx`.
pub fn serve(hub: Arc<Hub>, tx: Sender<Request>) -> std::io::Result<()> {
    let path = socket_path();
    if UnixStream::connect(&path).is_ok() {
        return Err(std::io::Error::new(std::io::ErrorKind::AddrInUse, "another coreglowd is running"));
    }
    let _ = std::fs::remove_file(&path);
    let listener = UnixListener::bind(&path)?;
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let (hub, tx) = (hub.clone(), tx.clone());
            std::thread::spawn(move || {
                let _ = handle(stream, &hub, &tx);
            });
        }
    });
    Ok(())
}

fn handle(stream: UnixStream, hub: &Hub, tx: &Sender<Request>) -> std::io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    let mut line = String::new();
    BufReader::new(&stream).read_line(&mut line)?;
    let mut out = &stream;
    let line = line.trim();
    let (cmd, arg) = line.split_once(' ').unwrap_or((line, ""));
    let reply = match cmd {
        "status" => hub.latest(),
        "test-fans" => {
            let _ = paths::request_fan_test(); // older clients and a restarted daemon see it too
            match tx.send(Request::TestFans) {
                Ok(()) => "ok".into(),
                Err(_) => "error: coreglowd is stopping".into(),
            }
        }
        "set" => set_profile(arg.trim()),
        "watch" => {
            let mut seen = 0;
            loop {
                let (n, status) = hub.next(seen);
                if n != seen {
                    seen = n;
                    writeln!(out, "{status}")?;
                }
            }
        }
        _ => format!("error: unknown request '{cmd}' (use set, test-fans, status, watch)"),
    };
    writeln!(out, "{reply}")
}

/// Write the active file; coreglowd sees the change on its next frame.
fn set_profile(name: &str) -> String {
    match crate::config::Config::load() {
        Ok(cfg) if cfg.profiles.contains_key(name) => match std::fs::write(paths::active_file(), format!("{name}\n")) {
            Ok(()) => "ok".into(),
            Err(e) => format!("error: {e}"),
        },
        Ok(_) => format!("error: no profile '{name}'"),
        Err(e) => format!("error: {e}"),
    }
}

// ---- clients ----

/// Send one request and read the one-line reply. Err when coreglowd isn't listening.
pub fn request(line: &str) -> Result<String, String> {
    let mut s = UnixStream::connect(socket_path()).map_err(|e| e.to_string())?;
    s.set_read_timeout(Some(Duration::from_secs(3))).map_err(|e| e.to_string())?;
    writeln!(s, "{line}").map_err(|e| e.to_string())?;
    let mut reply = String::new();
    BufReader::new(&s).read_line(&mut reply).map_err(|e| e.to_string())?;
    let reply = reply.trim().to_string();
    match reply.strip_prefix("error: ") {
        Some(e) => Err(e.to_string()),
        None => Ok(reply),
    }
}

/// Use a profile: through coreglowd when it listens, else by writing the active file.
pub fn set_profile_or_file(name: &str) -> Result<(), String> {
    match request(&format!("set {name}")) {
        Ok(_) => Ok(()),
        Err(e) if UnixStream::connect(socket_path()).is_ok() => Err(e),
        Err(_) => std::fs::write(paths::active_file(), format!("{name}\n")).map_err(|e| e.to_string()),
    }
}

/// Test the fans: through coreglowd when it listens, else with the request file.
pub fn test_fans() -> Result<(), String> {
    request("test-fans").map(drop).or_else(|_| paths::request_fan_test().map_err(|e| e.to_string()))
}

/// The latest status: from coreglowd when it listens, else the status file.
pub fn status() -> Option<Status> {
    request("status").ok().and_then(|l| serde_json::from_str(&l).ok()).or_else(Status::read)
}

/// Call `on_status` for every status coreglowd publishes, reconnecting as needed.
/// Runs forever (use a thread). While coreglowd isn't listening it reads the file every second.
pub fn watch(mut on_status: impl FnMut(Status)) -> ! {
    loop {
        if let Ok(mut s) = UnixStream::connect(socket_path()) {
            if writeln!(s, "watch").is_ok() {
                for line in BufReader::new(&s).lines() {
                    let Ok(line) = line else { break };
                    if let Ok(st) = serde_json::from_str(&line) {
                        on_status(st);
                    }
                }
            }
        }
        if let Some(st) = Status::read() {
            on_status(st);
        }
        std::thread::sleep(Duration::from_secs(1));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn watchers_wake_on_each_new_status() {
        let hub = Arc::new(Hub::default());
        let h = hub.clone();
        let waiter = std::thread::spawn(move || h.next(0));
        std::thread::sleep(Duration::from_millis(50));
        hub.publish(&Status { profile: "white".into(), ..Default::default() });
        let (n, line) = waiter.join().unwrap();
        assert_eq!(n, 1);
        assert!(line.contains("\"profile\":\"white\""));
        assert_eq!(hub.latest(), line);
    }
}
