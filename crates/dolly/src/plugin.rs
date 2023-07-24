use bevy::prelude::*;

use crate::prelude::CameraRig;

/// A `Resource` for controlling [`DollyPlugin`]
#[derive(Resource)]
pub struct DollySettings {}

impl Default for DollySettings {
    fn default() -> Self {
        Self {}
    }
}

/// Adds a system that syncs the Camera Transform when a CameraRig is initially added to it
pub struct DollyPlugin;

impl Plugin for DollyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DollySettings>().add_systems(Update, on_add_rig);
    }
}

fn on_add_rig(
    time: Res<Time>,
    mut q: Query<&mut CameraRig, (Without<Camera>, Added<CameraRig>)>,
    mut q2: Query<&mut Transform, With<Camera>>,
) {
    for mut rig in q.iter_mut() {
        let mut cam_transform = q2.single_mut();
        rig.update_into(time.delta_seconds(), cam_transform.as_mut());
    }
}
