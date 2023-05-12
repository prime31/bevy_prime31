use bevy::prelude::*;

pub use self::components::*;
use self::systems::*;

mod components;
mod systems;

#[derive(SystemSet, Clone, PartialEq, Eq, Debug, Hash)]
pub struct FpsControllerSystemSet;

#[derive(SystemSet, Clone, PartialEq, Eq, Debug, Hash)]
pub enum FpsControllerStages {
    Input,
    Logic,
    RenderSync,
}

#[derive(Default)]
pub struct FpsInputPlugin;

impl Plugin for FpsInputPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            (
                FpsControllerStages::Input,
                FpsControllerStages::Logic,
                FpsControllerStages::RenderSync,
            )
                .chain()
                .in_set(FpsControllerSystemSet),
        );

        app.register_type::<FpsControllerInput>()
            .register_type::<FpsControllerInputConfig>()
            .add_system(setup.on_startup().in_base_set(StartupSet::PostStartup))
            .add_system(controller_input.in_set(FpsControllerStages::Input))
            .add_system(calculate_movement)
            .add_system(sync_render_player.in_set(FpsControllerStages::RenderSync));
    }
}
