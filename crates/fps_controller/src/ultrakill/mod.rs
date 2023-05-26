use crate::input::FpsControllerStages;
use bevy::prelude::*;

pub use self::components::*;
use self::systems::*;

mod components;
mod systems;

#[derive(Default)]
pub struct UltrakillControllerPlugin;

impl Plugin for UltrakillControllerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<FpsControllerState>()
            .add_system(controller_move.in_set(FpsControllerStages::Logic))
            .add_system(debug_ui.run_if(egui_helper::run_if_egui_enabled));
    }
}
