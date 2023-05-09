use bevy::{prelude::*};
use bevy_inspector_egui::{quick::WorldInspectorPlugin, bevy_egui::{EguiSet, EguiContext}};

pub use bevy_inspector_egui;

#[derive(Resource, PartialEq, Eq)]
pub struct EguiHelperState {
    pub enabled: bool,
    pub wants_input: bool,
}

/// adds the WorldInspectorPlugin to the App and lets you hide/show it via pressing tilde. If egui wnats input
/// EguiHelperState.wants_input will be true and the game can choose to ignore input events.
#[derive(Default)]
pub struct EguiHelperPlugin;

impl Plugin for EguiHelperPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EguiHelperState {
            enabled: false,
            wants_input: false,
        })
        .add_plugin(
            WorldInspectorPlugin::new().run_if(move |res: Option<Res<EguiHelperState>>| match res {
                Some(res) => res.enabled,
                None => true,
            })
        ).add_system(update.after(EguiSet::ProcessInput).in_base_set(CoreSet::PreUpdate));
    }
}

fn update(mut q: Query<&mut EguiContext>, mut state: ResMut<EguiHelperState>, keyboard_input: Res<Input<KeyCode>>,) {
    for egui in q.iter_mut() {
        state.wants_input = egui.clone().get_mut().wants_pointer_input() || egui.clone().get_mut().wants_keyboard_input();
    }

    if keyboard_input.just_pressed(KeyCode::Grave) {
        state.enabled = !state.enabled;
    }
}
