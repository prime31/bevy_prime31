use bevy::prelude::*;

use self::loader::ValveMapLoader;

pub mod loader;

#[derive(Default)]
pub struct ValveMapPlugin;

impl Plugin for ValveMapPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<ValveMapLoader>();
    }
}