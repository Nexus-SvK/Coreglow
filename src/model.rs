//! The controllers coreglow drives. Both speak Bragi with the same files; they differ
//! in their ports, where the pump is, and whether a new lighting layout needs the
//! apply command. Facts from iCUE's device manifests (docs/corsair-devices.md,
//! icue-reverse-engineering.md §8c).

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Model {
    /// iCUE Commander CORE XT (stand-alone controller; this build's).
    #[default]
    CoreXt,
    /// Commander CORE, the hub of the ELITE CAPELLIX and ELITE LCD coolers. Experimental:
    /// written from the manifest, not yet tried on the hardware.
    Core,
}

impl Model {
    /// From a USB product ID. The bootloader IDs (0x0c1d, 0x0c3c) aren't included.
    pub fn from_pid(pid: u32) -> Option<Model> {
        match pid {
            0x0c2a => Some(Model::CoreXt),
            0x0c1c | 0x0c32 => Some(Model::Core),
            _ => None,
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Model::CoreXt => "core-xt",
            Model::Core => "core",
        }
    }

    pub fn from_id(id: &str) -> Option<Model> {
        [Model::CoreXt, Model::Core].into_iter().find(|m| m.id() == id)
    }

    pub fn name(self) -> &'static str {
        match self {
            Model::CoreXt => "Commander CORE XT",
            Model::Core => "Commander CORE",
        }
    }

    pub fn experimental(self) -> bool {
        self == Model::Core
    }

    /// RGB port names in controller order. The Core's port 0 is the pump's own ring.
    pub fn ports(self) -> &'static [&'static str; 7] {
        match self {
            Model::CoreXt => &["external", "port1", "port2", "port3", "port4", "port5", "port6"],
            Model::Core => &["pump", "port1", "port2", "port3", "port4", "port5", "port6"],
        }
    }

    /// Which model a [layout] table is written for (by its port names).
    pub fn for_ports<'a>(mut ports: impl Iterator<Item = &'a String>) -> Model {
        if ports.any(|p| p == "pump") {
            Model::Core
        } else {
            Model::CoreXt
        }
    }

    /// Cooling channel of fan port `n` (1-6).
    pub fn fan_channel(self, n: usize) -> usize {
        match self {
            Model::CoreXt => n - 1,
            Model::Core => n,
        }
    }

    /// Cooling channel of the pump: the Core's own pump port, or fan port 6 on this build's Core XT.
    pub fn pump_channel(self) -> usize {
        match self {
            Model::CoreXt => 5,
            Model::Core => 0,
        }
    }

    /// The Core takes fixed pump speeds (`UseFixedCoolingValuesForPump`); the Core XT's
    /// pump on a fan port runs at full speed in software mode whatever it's sent.
    pub fn pump_settable(self) -> bool {
        self == Model::Core
    }

    /// `RequiresApplyGeometryCommand` in the manifest: only the Core XT.
    pub fn needs_apply(self) -> bool {
        self == Model::CoreXt
    }

    /// Fan channels the hardware fallback curve drives: this build's two radiator fans
    /// on the Core XT (port 6 is its pump), all six fan ports on the Core.
    pub fn fallback_fans(self) -> Vec<usize> {
        match self {
            Model::CoreXt => vec![0, 1],
            Model::Core => (1..=6).collect(),
        }
    }
}
