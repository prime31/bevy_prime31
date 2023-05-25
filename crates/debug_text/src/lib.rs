#![doc = include_str!("../Readme.md")]

use bevy::prelude::Resource;

mod block;
mod overlay;
pub use overlay::{CommandChannels, InvocationSiteKey, DebugTextPlugin, COMMAND_CHANNELS};

/// Control position on screen of the debug overlay.
#[derive(Resource, Default)]
pub struct DebugOverlayLocation {
    pub margin_vertical: f32,
    pub margin_horizontal: f32,
}
