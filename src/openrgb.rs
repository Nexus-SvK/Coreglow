//! Minimal OpenRGB SDK client (network protocol version 0), used to drive the
//! motherboard lighting from the same effects as the Corsair controller.
//!
//! Packet: "ORGB", device index u32, packet id u32, data size u32, data (little endian).
//! Only what Coreglow needs: list controllers with their LED counts, switch a
//! controller to its direct ("custom") mode, and set all its LEDs at once.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

pub const DEFAULT_PORT: u16 = 6742;

const REQUEST_CONTROLLER_COUNT: u32 = 0;
const REQUEST_CONTROLLER_DATA: u32 = 1;
const SET_CLIENT_NAME: u32 = 50;
const UPDATE_LEDS: u32 = 1050;
const SET_CUSTOM_MODE: u32 = 1100;

#[derive(Clone, Debug)]
pub struct Controller {
    pub index: u32,
    pub name: String,
    pub leds: usize,
}

pub struct Client {
    stream: TcpStream,
    pub controllers: Vec<Controller>,
}

type Result<T> = std::result::Result<T, String>;

/// Reads the version-0 controller description far enough to count its LEDs.
struct Cursor<'a> {
    d: &'a [u8],
    p: usize,
}

impl Cursor<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8]> {
        let s = self.d.get(self.p..self.p + n).ok_or("controller data is shorter than expected")?;
        self.p += n;
        Ok(s)
    }
    fn u16(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }
    fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn string(&mut self) -> Result<String> {
        let n = self.u16()? as usize;
        let b = self.take(n)?;
        Ok(String::from_utf8_lossy(b).trim_end_matches('\0').to_string())
    }
}

fn parse_controller(index: u32, data: &[u8]) -> Result<Controller> {
    let mut c = Cursor { d: data, p: 0 };
    c.u32()?; // data size
    c.u32()?; // device type
    let name = c.string()?;
    for _ in 0..4 {
        c.string()?; // description, version, serial, location
    }
    let modes = c.u16()?;
    c.u32()?; // active mode
    for _ in 0..modes {
        c.string()?; // name
        for _ in 0..9 {
            c.u32()?; // value, flags, speed min/max, colors min/max, speed, direction, color mode
        }
        let colors = c.u16()? as usize;
        c.take(colors * 4)?;
    }
    let zones = c.u16()?;
    for _ in 0..zones {
        c.string()?;
        for _ in 0..4 {
            c.u32()?; // type, leds min/max/count
        }
        let matrix = c.u16()? as usize;
        c.take(matrix)?;
    }
    let leds = c.u16()? as usize;
    Ok(Controller { index, name, leds })
}

impl Client {
    pub fn connect(host: &str, port: u16) -> Result<Client> {
        let addr = format!("{host}:{port}");
        let sock = std::net::ToSocketAddrs::to_socket_addrs(&addr)
            .map_err(|e| format!("OpenRGB {addr}: {e}"))?
            .next()
            .ok_or(format!("OpenRGB {addr}: no address"))?;
        let stream = TcpStream::connect_timeout(&sock, Duration::from_millis(500))
            .map_err(|e| format!("OpenRGB server at {addr} isn't reachable ({e}); is `openrgb --server` running?"))?;
        stream.set_read_timeout(Some(Duration::from_secs(2))).ok();
        stream.set_write_timeout(Some(Duration::from_secs(2))).ok();
        stream.set_nodelay(true).ok();
        let mut client = Client { stream, controllers: Vec::new() };
        client.send(0, SET_CLIENT_NAME, b"Coreglow\0")?;
        client.send(0, REQUEST_CONTROLLER_COUNT, &[])?;
        let count = u32::from_le_bytes(client.reply(REQUEST_CONTROLLER_COUNT)?.get(..4).ok_or("short reply")?.try_into().unwrap());
        for i in 0..count {
            client.send(i, REQUEST_CONTROLLER_DATA, &[])?;
            let data = client.reply(REQUEST_CONTROLLER_DATA)?;
            client.controllers.push(parse_controller(i, &data)?);
        }
        Ok(client)
    }

    fn send(&mut self, dev: u32, id: u32, data: &[u8]) -> Result<()> {
        let mut pkt = Vec::with_capacity(16 + data.len());
        pkt.extend_from_slice(b"ORGB");
        pkt.extend_from_slice(&dev.to_le_bytes());
        pkt.extend_from_slice(&id.to_le_bytes());
        pkt.extend_from_slice(&(data.len() as u32).to_le_bytes());
        pkt.extend_from_slice(data);
        self.stream.write_all(&pkt).map_err(|e| format!("OpenRGB: {e}"))
    }

    /// Next packet with this id; others (e.g. device-list-updated notices) are skipped.
    fn reply(&mut self, id: u32) -> Result<Vec<u8>> {
        for _ in 0..16 {
            let mut head = [0u8; 16];
            self.stream.read_exact(&mut head).map_err(|e| format!("OpenRGB: {e}"))?;
            if &head[..4] != b"ORGB" {
                return Err("OpenRGB: bad reply".into());
            }
            let got = u32::from_le_bytes(head[8..12].try_into().unwrap());
            let size = u32::from_le_bytes(head[12..16].try_into().unwrap()) as usize;
            let mut data = vec![0u8; size];
            self.stream.read_exact(&mut data).map_err(|e| format!("OpenRGB: {e}"))?;
            if got == id {
                return Ok(data);
            }
        }
        Err("OpenRGB: no reply".into())
    }

    /// Put a controller in its direct mode, so colours sent next show as they are.
    pub fn set_custom_mode(&mut self, dev: u32) -> Result<()> {
        self.send(dev, SET_CUSTOM_MODE, &[])
    }

    pub fn set_leds(&mut self, dev: u32, colors: &[[u8; 3]]) -> Result<()> {
        let mut data = Vec::with_capacity(6 + colors.len() * 4);
        data.extend_from_slice(&((6 + colors.len() * 4) as u32).to_le_bytes());
        data.extend_from_slice(&(colors.len() as u16).to_le_bytes());
        for c in colors {
            data.extend_from_slice(&[c[0], c[1], c[2], 0]);
        }
        self.send(dev, UPDATE_LEDS, &data)
    }

    /// LEDs across all controllers, in controller order.
    pub fn total_leds(&self) -> usize {
        self.controllers.iter().map(|c| c.leds).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(out: &mut Vec<u8>, v: &str) {
        out.extend_from_slice(&((v.len() + 1) as u16).to_le_bytes());
        out.extend_from_slice(v.as_bytes());
        out.push(0);
    }

    #[test]
    fn parses_a_version_0_controller() {
        let mut d = vec![0, 0, 0, 0, 4, 0, 0, 0]; // size (unused), type
        s(&mut d, "B650 AORUS ELITE AX V2");
        for v in ["desc", "1.0", "", "HID: /dev/hidraw0"] {
            s(&mut d, v);
        }
        d.extend_from_slice(&1u16.to_le_bytes()); // one mode
        d.extend_from_slice(&0u32.to_le_bytes()); // active
        s(&mut d, "Direct");
        d.extend_from_slice(&[0u8; 36]); // nine u32 fields
        d.extend_from_slice(&1u16.to_le_bytes()); // one mode colour
        d.extend_from_slice(&[0u8; 4]);
        d.extend_from_slice(&1u16.to_le_bytes()); // one zone
        s(&mut d, "Motherboard");
        d.extend_from_slice(&[0u8; 16]);
        d.extend_from_slice(&0u16.to_le_bytes()); // no matrix
        d.extend_from_slice(&2u16.to_le_bytes()); // two LEDs
        let c = parse_controller(0, &d).unwrap();
        assert_eq!((c.name.as_str(), c.leds), ("B650 AORUS ELITE AX V2", 2));
    }
}
