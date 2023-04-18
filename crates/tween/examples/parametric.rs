use std::time::Duration;

use bevy::prelude::*;
use tween::{Animator, EaseFunction, Lens, Tween, TweeningPlugin};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TransformParametricPositionLens {
    pub start: Vec3,
}

impl Lens<Transform> for TransformParametricPositionLens {
    fn lerp(&mut self, target: &mut Transform, ratio: f32) {
        target.translation.x = self.start.x + 3. * (ratio * 6.28).cos();
        target.translation.y = self.start.y + ratio * 4.;
        target.translation.z = self.start.z + 3. * (ratio * 6.28).sin();
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(TweeningPlugin)
        .add_plugin(cameras::pan_orbit::PanOrbitCameraPlugin)
        .add_startup_system(setup)
        .add_startup_system(setup_tween)
        .run();
}

/// set up a simple 3D scene
fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    // plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(25.0).into()),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn setup_tween(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let tween = Tween::new(
        EaseFunction::QuadraticInOut,
        Duration::from_millis(4000),
        TransformParametricPositionLens {
            start: Vec3::new(0.0, 1.5, 0.0),
        },
    )
    .with_repeat_count(3)
    .with_repeat_strategy(tween::RepeatStrategy::MirroredRepeat);

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 0.5 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 1.5, 0.0),
            ..default()
        },
        Animator::new(tween),
    ));
}
