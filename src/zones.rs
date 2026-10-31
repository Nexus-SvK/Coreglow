//! Lighting layout of this build and the named parts ("zones") of it.
//!
//! LightingChannelsGeometries (Bragi file 0x1e): channel count, then per channel a
//! device count followed by one device-type byte per device. This is the layout
//! iCUE found stored in the controller: external port = 6 type-06 slots (4 LED
//! strips use the first 40 LEDs), ports 1-2 = fans (05), port 3 = pump head (0c).
//! Which part is where was mapped with a colour test.

use std::ops::Range;

pub const PORT_CONFIG: [u8; 17] = [
    0x07, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x01, 0x05, 0x01, 0x05, 0x01, 0x0c, 0x00, 0x00, 0x00,
];

/// LEDs per device type, from iCUE's frame sizes (6 x 34 + 8 + 8 + 13 = 233).
fn type_leds(t: u8) -> usize {
    match t {
        0x06 => 34,
        0x05 => 8,
        0x0c => 13,
        _ => 0,
    }
}

const STRIPS: usize = 4;
const STRIP_LEDS: usize = 10;

pub const ZONE_NAMES: [&str; 10] =
    ["strip1", "strip2", "strip3", "strip4", "strips", "right-fan", "left-fan", "fans", "pump", "all"];

/// Parts shown in the live picture (one colour each).
pub const PICTURE_ZONES: [&str; 7] = ["strip1", "strip2", "strip3", "strip4", "right-fan", "left-fan", "pump"];

/// Frame slice of each channel, in frame order.
pub fn channel_ranges() -> Vec<Range<usize>> {
    let mut out = Vec::new();
    let (mut start, mut pos) = (0, 1);
    for _ in 0..PORT_CONFIG[0] {
        let n = PORT_CONFIG[pos] as usize;
        let size: usize = PORT_CONFIG[pos + 1..pos + 1 + n].iter().map(|&t| type_leds(t)).sum();
        out.push(start..start + size);
        start += size;
        pos += 1 + n;
    }
    out
}

/// Number of LEDs in a lighting frame (including slots nothing is connected to).
pub fn frame_len() -> usize {
    channel_ranges().last().map_or(0, |r| r.end)
}

/// Frame indices of the LEDs in a zone. `all` runs strips -> right fan -> left fan
/// -> pump, so moving effects flow across the parts without gaps.
pub fn zone(name: &str) -> Vec<usize> {
    let ch = channel_ranges();
    let ext = ch[0].start;
    let strip = |i: usize| (ext + i * STRIP_LEDS..ext + (i + 1) * STRIP_LEDS).collect::<Vec<_>>();
    match name {
        "strip1" => strip(0),
        "strip2" => strip(1),
        "strip3" => strip(2),
        "strip4" => strip(3),
        "strips" => (ext..ext + STRIPS * STRIP_LEDS).collect(),
        "right-fan" => ch[1].clone().collect(),
        "left-fan" => ch[2].clone().collect(),
        "fans" => ch[1].clone().chain(ch[2].clone()).collect(),
        "pump" => ch[3].clone().collect(),
        "all" => ["strips", "fans", "pump"].iter().flat_map(|z| zone(z)).collect(),
        _ => Vec::new(),
    }
}

pub fn label(name: &str) -> &'static str {
    match name {
        "strip1" => "LED strip 1",
        "strip2" => "LED strip 2",
        "strip3" => "LED strip 3",
        "strip4" => "LED strip 4",
        "strips" => "All LED strips",
        "right-fan" => "Right fan",
        "left-fan" => "Left fan",
        "fans" => "Both radiator fans",
        "pump" => "Pump head",
        _ => "Everything",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_matches_icue() {
        assert_eq!(frame_len(), 233);
        assert_eq!(zone("all").len(), 69);
        assert_eq!(zone("right-fan"), (204..212).collect::<Vec<_>>());
        assert_eq!(zone("left-fan"), (212..220).collect::<Vec<_>>());
        assert_eq!(zone("pump"), (220..233).collect::<Vec<_>>());
    }
}
