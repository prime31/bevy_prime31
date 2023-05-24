use bevy::prelude::*;
use bevy_prototype_debug_lines::DebugLinesPlugin;
use leafwing_input_manager::prelude::InputManagerPlugin;

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

        app.add_plugin(InputManagerPlugin::<InputAction>::default())
            .add_plugin(DebugLinesPlugin::with_depth_test(true))
            .register_type::<FpsControllerInput>()
            .register_type::<FpsControllerInputConfig>()
            .add_system(setup.on_startup().in_base_set(StartupSet::PostStartup))
            .add_systems((controller_input, sync_rotation_input, temp_input_test).in_set(FpsControllerStages::Input));
    }
}
