use std::time::Duration;

use bevy::prelude::*;
use tween::{
    lens::TransformPositionLens, unit_sphere, Animator, Delay, EaseFunction, Tween, TweeningPlugin,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(TweeningPlugin)
        .add_plugin(bevy::diagnostic::LogDiagnosticsPlugin::default())
        .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
        .add_startup_system(setup)
        .add_startup_system(setup_tween)
        .add_startup_system(setup_tween_sequence)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
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
        EaseFunction::ExponentialOut,
        Duration::from_millis(2000),
        TransformPositionLens {
            start: Vec3::new(0.0, 0.5, 0.0),
            end: Vec3::new(0.0, 2.5, 0.0),
        },
    )
    .with_repeat_count(3)
    .with_repeat_strategy(tween::RepeatStrategy::MirroredRepeat);

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        },
        Animator::new(tween),
    ));
}

fn setup_tween_sequence(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let seq = Delay::new(Duration::from_millis(500))
        .then(Tween::new(
            EaseFunction::QuadraticIn,
            Duration::from_millis(250),
            TransformPositionLens {
                start: Vec3::new(1.5, 0.5, 0.0),
                end: Vec3::new(1.5, 2.5, 0.0),
            },
        ))
        .then(Tween::new(
            EaseFunction::QuadraticIn,
            Duration::from_millis(250),
            TransformPositionLens {
                start: Vec3::new(1.5, 2.5, 0.0),
                end: Vec3::new(2.5, 2.5, 0.0),
            },
        ));

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 0.5 })),
            material: materials.add(Color::BLUE.into()),
            transform: Transform::from_xyz(1.5, 0.5, 0.0),
            ..default()
        },
        Animator::new(seq),
    ));

    //
    let total_projectiles = 500;
    let initial_delay = 5;
    let tween_duration = 150;
    let total_duration = (total_projectiles * initial_delay) + (tween_duration * 2) + 1;
    for i in 0..total_projectiles {
        let pt = unit_sphere::sample_hemisphere(&mut rand::thread_rng());
        if pt.y < 0.0 {
            continue;
        }

        let delay = i * 5 + 1;
        let seq = Delay::new(Duration::from_millis(delay))
            .then(Tween::new(
                EaseFunction::QuadraticIn,
                Duration::from_millis(tween_duration),
                TransformPositionLens {
                    start: Vec3::ZERO,
                    end: pt * 3.,
                },
            ))
            .then(
                Tween::new(
                    EaseFunction::QuadraticOut,
                    Duration::from_millis(tween_duration),
                    TransformPositionLens {
                        start: pt * 3.,
                        end: pt * 4.,
                    },
                )
                .then(
                    Delay::new(Duration::from_millis(total_duration - delay)).then(Tween::new(
                        EaseFunction::QuadraticOut,
                        Duration::from_millis(250),
                        TransformPositionLens {
                            start: pt * 6.,
                            end: Vec3::new(-2.0, 2.5, 15.0),
                        },
                    )),
                ),
            );

        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 0.2 })),
                material: materials.add(Color::RED.into()),
                ..default()
            },
            Animator::new(seq),
        ));
    }
}
