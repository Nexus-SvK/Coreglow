//! Commander Core XT over Linux hidraw, Bragi protocol.
//!
//! Every command is one 384-byte output report: `08 <cmd> <args…>`; the device
//! answers with one input report `00 <cmd> <status> …`. Data lives in numbered
//! "files" that are opened, read or written in chunks, and closed. 384-byte
//! reports matter: with liquidctl's 96-byte reports, long lighting frames land in
//! the wrong place.

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::os::fd::AsRawFd;
use std::path::PathBuf;

use crate::zones::Layout;

pub type Result<T> = std::result::Result<T, String>;

const VENDOR: u32 = 0x1b1c;
const PRODUCT: u32 = 0x0c2a;
const REPORT: usize = 384;
const TIMEOUT_MS: i32 = 1000;

const CMD_WAKE: [u8; 4] = [0x01, 0x03, 0x00, 0x02]; // property 3 (mode) = 2: software mode
const CMD_SLEEP: [u8; 4] = [0x01, 0x03, 0x00, 0x01]; // = 1: hardware mode
const CMD_OPEN: [u8; 2] = [0x0d, 0x00];
const CMD_CLOSE: [u8; 3] = [0x05, 0x01, 0x00];
const CMD_READ_INITIAL: [u8; 3] = [0x08, 0x00, 0x01];
const CMD_READ_MORE: [u8; 3] = [0x08, 0x00, 0x02];
const CMD_READ_FINAL: [u8; 3] = [0x08, 0x00, 0x03];
const CMD_WRITE: [u8; 2] = [0x06, 0x00];
const CMD_WRITE_MORE: [u8; 2] = [0x07, 0x00];
/// iCUE sends this after writing the lighting layout; it applies it.
const CMD_APPLY_LIGHTING: [u8; 2] = [0x15, 0x01];

// Bragi file IDs (bragiprotocol::FileId) and their data types
const F_COOLERS_RPM: &[u8] = &[0x17];
const T_COOLERS_RPM: [u8; 2] = [0x06, 0x00];
const F_TARGET_ROTATION: &[u8] = &[0x18]; // software-mode fan duties
const T_TARGET_ROTATION: [u8; 2] = [0x07, 0x00];
const F_STRIP_QUANTITIES: &[u8] = &[0x1d]; // LightingChannelsControllersQuantity
const T_STRIP_QUANTITIES: [u8; 2] = [0x0c, 0x00];
const F_AUTODETECTED: &[u8] = &[0x20]; // LightingChannelsAutodetectionResults
const T_AUTODETECTED: [u8; 2] = [0x0f, 0x00];
const F_GEOMETRIES: &[u8] = &[0x1e];
const T_GEOMETRIES: [u8; 2] = [0x0d, 0x00];
const F_TEMPERATURES: &[u8] = &[0x21];
const T_TEMPERATURES: [u8; 2] = [0x10, 0x00];
const F_LIGHTING_BUFFER: &[u8] = &[0x22];
const T_LIGHTING_BUFFER: [u8; 2] = [0x12, 0x00];
// Hardware-mode cooling settings live in free files 0x6d60.. (as liquidctl uses them)
const F_HW_MODE: &[u8] = &[0x60, 0x6d];
const T_HW_MODE: [u8; 2] = [0x03, 0x00];
const F_HW_FIXED: &[u8] = &[0x61, 0x6d];
const T_HW_FIXED: [u8; 2] = [0x04, 0x00];
const F_HW_CURVE: &[u8] = &[0x62, 0x6d];
const T_HW_CURVE: [u8; 2] = [0x05, 0x00];
const HW_FIXED_PERCENT: u8 = 0x00;
const HW_CURVE_PERCENT: u8 = 0x02;

/// Channel of the AIO pump (fan port 6).
pub const PUMP_CHANNEL: usize = 5;

pub struct Device {
    file: File,
    pub path: PathBuf,
}

/// Find the controller's hidraw node (USB interface 0).
pub fn find() -> Option<PathBuf> {
    let want = format!("HID_ID=0003:{VENDOR:08X}:{PRODUCT:08X}");
    let mut nodes: Vec<_> = std::fs::read_dir("/sys/class/hidraw").ok()?.flatten().collect();
    nodes.sort_by_key(|e| e.file_name());
    for e in nodes {
        let uevent = std::fs::read_to_string(e.path().join("device/uevent")).unwrap_or_default();
        if !uevent.lines().any(|l| l == want) {
            continue;
        }
        let real = std::fs::canonicalize(e.path().join("device")).unwrap_or_default();
        if real.to_string_lossy().contains(":1.0/") {
            return Some(PathBuf::from("/dev").join(e.file_name()));
        }
    }
    None
}

impl Device {
    pub fn open() -> Result<Device> {
        let path = find().ok_or("Commander Core XT not found (is it plugged in, or passed to a VM?)")?;
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .map_err(|e| format!("{}: {e} (is the udev rule installed and are you logged in at the seat?)", path.display()))?;
        Ok(Device { file, path })
    }

    fn readable(&self, timeout_ms: i32) -> Result<bool> {
        let mut p = libc::pollfd { fd: self.file.as_raw_fd(), events: libc::POLLIN, revents: 0 };
        let r = unsafe { libc::poll(&mut p, 1, timeout_ms) };
        if r < 0 {
            return Err(std::io::Error::last_os_error().to_string());
        }
        if p.revents & (libc::POLLERR | libc::POLLHUP | libc::POLLNVAL) != 0 {
            return Err("device went away".into());
        }
        Ok(r > 0)
    }

    /// One command, one reply. Replies left over from earlier commands are dropped.
    fn send(&mut self, cmd: &[u8], data: &[u8]) -> Result<Vec<u8>> {
        let mut junk = [0u8; 1024];
        while self.readable(0)? {
            let _ = self.file.read(&mut junk);
        }
        let mut buf = vec![0u8; REPORT + 1]; // [0] = report id (unused)
        buf[1] = 0x08;
        buf[2..2 + cmd.len()].copy_from_slice(cmd);
        let n = (REPORT + 1 - 2 - cmd.len()).min(data.len());
        buf[2 + cmd.len()..2 + cmd.len() + n].copy_from_slice(&data[..n]);
        self.file.write_all(&buf).map_err(|e| format!("write: {e}"))?;
        for _ in 0..16 {
            if !self.readable(TIMEOUT_MS)? {
                return Err(format!("no reply to command {:02x}", cmd[0]));
            }
            let mut resp = vec![0u8; 1024];
            let got = self.file.read(&mut resp).map_err(|e| format!("read: {e}"))?;
            resp.truncate(got);
            if resp.len() >= 3 && resp[0] == 0x00 && resp[1] == cmd[0] {
                return Ok(resp);
            }
        }
        Err(format!("unexpected replies to command {:02x}", cmd[0]))
    }

    fn open_file(&mut self, file: &[u8]) -> Result<()> {
        self.send(&CMD_OPEN, file).map(drop)
    }

    fn close_file(&mut self) -> Result<()> {
        self.send(&CMD_CLOSE, &[]).map(drop)
    }

    pub fn wake(&mut self) -> Result<()> {
        self.send(&CMD_WAKE, &[]).map(drop)
    }

    pub fn sleep(&mut self) -> Result<()> {
        self.send(&CMD_SLEEP, &[]).map(drop)
    }

    fn read_data(&mut self, file: &[u8], dtype: [u8; 2]) -> Result<Vec<u8>> {
        self.open_file(file)?;
        let first = self.send(&CMD_READ_INITIAL, &[]);
        let more = self.send(&CMD_READ_MORE, &[]);
        let last = self.send(&CMD_READ_FINAL, &[]);
        self.close_file()?;
        let (first, more, last) = (first?, more?, last?);
        if first.len() < 5 || first[3..5] != dtype {
            return Err(format!("file {file:02x?}: unexpected data type"));
        }
        let mut out = first[5..].to_vec();
        out.extend_from_slice(&more[3..]);
        out.extend_from_slice(&last[3..]);
        Ok(out)
    }

    /// Write a whole file: length + data type in the first report, the rest in WRITE_MORE reports.
    fn write_data(&mut self, file: &[u8], dtype: [u8; 2], data: &[u8]) -> Result<()> {
        self.open_file(file)?;
        let first = REPORT - 9;
        let len = (data.len() + 2) as u16;
        let mut head = vec![len as u8, (len >> 8) as u8, 0, 0, dtype[0], dtype[1]];
        head.extend_from_slice(&data[..first.min(data.len())]);
        let mut result = self.send(&CMD_WRITE, &head).map(drop);
        let mut pos = first;
        while result.is_ok() && pos < data.len() {
            let end = (pos + REPORT - 3).min(data.len());
            result = self.send(&CMD_WRITE_MORE, &data[pos..end]).map(drop);
            pos = end;
        }
        let closed = self.close_file();
        result.and(closed)
    }

    // ---- software mode (daemon) ----

    /// Write the lighting layout and apply it, like iCUE does.
    pub fn init_lighting(&mut self, layout: &Layout) -> Result<()> {
        if let Some(q) = layout.strip_quantities() {
            self.write_data(F_STRIP_QUANTITIES, T_STRIP_QUANTITIES, &q)?;
        }
        self.write_data(F_GEOMETRIES, T_GEOMETRIES, &layout.geometry())?;
        self.send(&CMD_APPLY_LIGHTING, &[]).map(drop)
    }

    /// LEDs the controller detected on each RGB port: (status, LED count). Status 2 =
    /// something detected; the external port isn't auto-detected.
    pub fn autodetected(&mut self) -> Result<Vec<(u16, u16)>> {
        let d = self.read_data(F_AUTODETECTED, T_AUTODETECTED)?;
        let n = *d.first().unwrap_or(&0) as usize;
        Ok((0..n)
            .map(|i| {
                let o = 1 + 4 * i;
                (u16::from_le_bytes([d[o], d[o + 1]]), u16::from_le_bytes([d[o + 2], d[o + 3]]))
            })
            .collect())
    }

    pub fn write_frame(&mut self, rgb: &[u8]) -> Result<()> {
        self.write_data(F_LIGHTING_BUFFER, T_LIGHTING_BUFFER, rgb)
    }

    /// Duty in % per channel (0-based). Only works in software mode, and the pump ignores it.
    pub fn set_duties(&mut self, duties: &[(usize, u8)]) -> Result<()> {
        let mut data = vec![duties.len() as u8];
        for &(ch, d) in duties {
            data.extend_from_slice(&[ch as u8, 0x00, d, 0x00]);
        }
        self.write_data(F_TARGET_ROTATION, T_TARGET_ROTATION, &data)
    }

    /// Probe temperatures in °C (None = probe not connected).
    pub fn temps(&mut self) -> Result<Vec<Option<f64>>> {
        let d = self.read_data(F_TEMPERATURES, T_TEMPERATURES)?;
        let n = *d.first().unwrap_or(&0) as usize;
        Ok((0..n)
            .map(|i| {
                let o = 1 + 3 * i;
                (d.get(o) == Some(&0)).then(|| u16::from_le_bytes([d[o + 1], d[o + 2]]) as f64 / 10.0)
            })
            .collect())
    }

    pub fn speeds(&mut self) -> Result<Vec<u16>> {
        let d = self.read_data(F_COOLERS_RPM, T_COOLERS_RPM)?;
        let n = *d.first().unwrap_or(&0) as usize;
        Ok((0..n).map(|i| u16::from_le_bytes([d[1 + 2 * i], d[2 + 2 * i]])).collect())
    }

    // ---- hardware mode (fallback when no software is running) ----

    /// Store a fan curve (driven by probe 1) for `channels` and a fixed duty for
    /// `fixed` channels, used by the controller on its own in hardware mode.
    pub fn store_hardware_cooling(&mut self, channels: &[usize], curve: &[(f64, f64)], fixed: &[(usize, u8)]) -> Result<()> {
        let modes = self.read_data(F_HW_MODE, T_HW_MODE)?;
        let count = modes[0] as usize;
        let mut new_modes = modes[..=count].to_vec();
        for &ch in channels {
            new_modes[ch + 1] = HW_CURVE_PERCENT;
        }
        for &(ch, _) in fixed {
            new_modes[ch + 1] = HW_FIXED_PERCENT;
        }
        self.write_data(F_HW_MODE, T_HW_MODE, &new_modes)?;

        let fx = self.read_data(F_HW_FIXED, T_HW_FIXED)?;
        let mut new_fx = fx[..1 + 2 * fx[0] as usize].to_vec();
        for &(ch, d) in fixed {
            new_fx[1 + 2 * ch] = d;
            new_fx[2 + 2 * ch] = 0;
        }
        self.write_data(F_HW_FIXED, T_HW_FIXED, &new_fx)?;

        let cur = self.read_data(F_HW_CURVE, T_HW_CURVE)?;
        let devices = cur[0] as usize;
        let mut blocks = Vec::new();
        let mut i = 1;
        for _ in 0..devices {
            let points = cur[i + 1] as usize;
            let end = i + 2 + 4 * points;
            blocks.push(cur[i..end].to_vec());
            i = end;
        }
        let mut block = vec![0x00, curve.len() as u8]; // sensor 0 = probe 1
        for &(t, d) in curve {
            block.extend_from_slice(&((t * 10.0).round() as u16).to_le_bytes());
            block.extend_from_slice(&(d.clamp(0.0, 100.0).round() as u16).to_le_bytes());
        }
        for &ch in channels {
            blocks[ch] = block.clone();
        }
        let mut out = vec![devices as u8];
        blocks.iter().for_each(|b| out.extend_from_slice(b));
        self.write_data(F_HW_CURVE, T_HW_CURVE, &out)
    }
}

/// The controller's fallback: fans 1 and 2 on `curve` (probe 1), pump at 100 %.
pub fn store_fallback(dev: &mut Device, curve: &[(f64, f64)]) -> Result<()> {
    dev.store_hardware_cooling(&[0, 1], curve, &[(PUMP_CHANNEL, 100)])
}
