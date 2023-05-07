use bevy::prelude::*;

#[derive(Default)]
pub struct FPSControllerPlugin;

impl Plugin for FPSControllerPlugin {
    fn build(&self, _app: &mut App) {
        // app.init_asset_loader::<ValveMapLoader>()
        //     .add_asset::<ValveMap>()
        //     .add_system(handle_loaded_maps);
    }
}