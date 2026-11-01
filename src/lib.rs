//! Shared code for coreglowd (daemon), coreglowctl (CLI) and coreglow.
//!
//! The Commander CORE XT (and the Commander CORE) speak Corsair's Bragi protocol over USB HID. Everything
//! here was worked out from USB captures of iCUE and OpenRGB, liquidctl's
//! commander_core driver and iCUE's BragiProtocols.dll (file IDs from its
//! `bragiprotocol::FileId` enum).

pub mod auto;
pub mod config;
pub mod control;
pub mod device;
pub mod effects;
pub mod i18n;
pub mod model;
pub mod openrgb;
pub mod paths;
pub mod sensors;
pub mod status;
pub mod zones;
