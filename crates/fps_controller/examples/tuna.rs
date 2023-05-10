mod common;

use std::f32::consts::TAU;

use bevy::{prelude::*, render::{view::RenderLayers, camera::Viewport}, core_pipeline::clear_color::ClearColorConfig};
use bevy_rapier3d::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_tnua::{
    TnuaFreeFallBehavior, TnuaPlatformerAnimatingOutput, TnuaPlatformerBundle, TnuaPlatformerConfig,
    TnuaPlatformerControls, TnuaPlatformerPlugin, TnuaRapier3dPlugin,
};

use common::MovingPlatform;
use egui_helper::EguiHelperPlugin;
use valve_maps::bevy::{ValveMapBundle, ValveMapPlayer, ValveMapPlugin};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugin(RapierPhysicsPlugin::<NoUserData>::default());
    app.add_plugin(RapierDebugRenderPlugin::default());
    app.add_plugin(TnuaRapier3dPlugin);
    app.add_plugin(TnuaPlatformerPlugin);
    app.add_plugin(EguiHelperPlugin);
    app.add_plugin(ValveMapPlugin);
    app.add_startup_system(setup_camera);
    app.add_startup_system(setup_level);
    app.add_startup_system(setup_player);
    app.add_system(apply_controls);
    app.add_system(MovingPlatform::make_system(|velocity: &mut Velocity, linvel: Vec3| {
        velocity.linvel = linvel;
    }));
    app.run();
}

fn setup_camera(mut commands: Commands) {
    // commands.spawn(Camera3dBundle {
    //     transform: Transform::from_xyz(0.0, 16.0, 40.0).looking_at(Vec3::new(0.0, 10.0, 0.0), Vec3::Y),
    //     ..Default::default()
    // });

    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(5.0, 5.0, 5.0),
        ..default()
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 4000.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::default().looking_at(-Vec3::Y, Vec3::Z),
        ..Default::default()
    });
}

fn setup_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(ValveMapBundle {
        map: asset_server.load("playground.map"),
        ..Default::default()
    });

    let mut cmd = commands.spawn_empty();
    cmd.insert(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane {
            size: 128.0,
            subdivisions: 0,
        })),
        material: materials.add(Color::WHITE.into()),
        transform: Transform::from_translation(Vec3::new(0.0, -0.1, 0.0)),
        ..Default::default()
    });
    cmd.insert(Collider::halfspace(Vec3::Y).unwrap());

    // spawn moving platform
    {
        let mut cmd = commands.spawn_empty();

        cmd.insert(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(4.0, 1.0, 4.0))),
            material: materials.add(Color::BLUE.into()),
            transform: Transform::from_xyz(-4.0, 6.0, 0.0),
            ..Default::default()
        });
        cmd.insert(Collider::cuboid(2.0, 0.5, 2.0));
        cmd.insert(Velocity::default());
        cmd.insert(RigidBody::KinematicVelocityBased);
        cmd.insert(MovingPlatform::new(
            4.0,
            &[
                Vec3::new(-4.0, 6.0, 0.0),
                Vec3::new(-8.0, 6.0, 0.0),
                Vec3::new(-8.0, 10.0, 0.0),
                Vec3::new(-8.0, 10.0, -4.0),
                Vec3::new(-4.0, 10.0, -4.0),
                Vec3::new(-4.0, 10.0, 0.0),
            ],
        ));
    }

    // spawn spinning platform
    {
        let mut cmd = commands.spawn_empty();

        cmd.insert(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cylinder {
                radius: 3.0,
                height: 1.0,
                resolution: 10,
                segments: 10,
            })),
            material: materials.add(Color::BLUE.into()),
            transform: Transform::from_xyz(-2.0, 2.0, 10.0),
            ..Default::default()
        });
        cmd.insert(Collider::cylinder(0.5, 3.0));
        cmd.insert(Velocity::angular(Vec3::Y));
        cmd.insert(RigidBody::KinematicVelocityBased);
    }
}

fn setup_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut cmd = commands.spawn_empty();
    cmd.insert(ValveMapPlayer);
    cmd.insert(PbrBundle {
        mesh: meshes.add(shape::Capsule::default().into()),
        material: materials.add(Color::rgb(0.8, 0.1, 0.9).into()),
        ..Default::default()
    });
    cmd.insert(RigidBody::Dynamic);
    cmd.insert(Velocity::default());
    cmd.insert(Collider::capsule_y(0.5, 0.5));
    cmd.insert(TnuaPlatformerBundle::new_with_config(TnuaPlatformerConfig {
        full_speed: 20.0,
        full_jump_height: 6.0,
        up: Vec3::Y,
        forward: -Vec3::Z,
        float_height: 1.0,
        cling_distance: 1.0,
        spring_strengh: 400.0,
        spring_dampening: 1.2,
        acceleration: 60.0,
        air_acceleration: 40.0,
        coyote_time: 0.15,
        jump_start_extra_gravity: 30.0,
        jump_fall_extra_gravity: 40.0,
        jump_shorten_extra_gravity: 40.0,
        free_fall_behavior: TnuaFreeFallBehavior::LikeJumpShorten,
        tilt_offset_angvel: 5.0,
        tilt_offset_angacl: 500.0,
        turning_angvel: 10.0,
    }));
    cmd.insert(TnuaPlatformerAnimatingOutput::default());

    cmd.with_children(|builder| {
        builder
            .spawn((
                Camera3dBundle {
                    transform: Transform::from_xyz(0.0, 1.0, 0.0),
                    projection: Projection::Perspective(PerspectiveProjection {
                        fov: TAU / 5.0,
                        ..default()
                    }),
                    ..default()
                },
                RenderLayers::default().without(1), // all but our LogicalPlayer
            ))
            .with_children(|builder| {
                // Right Camera for 3rd person view trailing a bit and slightly above the player
                let win_w = 1280;
                let frame_w = 256;
                let frame_h = 256 / (1280 / 720);
                builder.spawn((
                    Camera3dBundle {
                        transform: Transform::from_xyz(0., 1.5, 15.0),
                        camera: Camera {
                            order: 1, // after other camera
                            viewport: Some(Viewport {
                                physical_position: UVec2::new(win_w * 2 - frame_w * 2, 0),
                                physical_size: UVec2::new(frame_w * 2, frame_h * 2),
                                ..default()
                            }),
                            ..default()
                        },
                        camera_3d: Camera3d {
                            clear_color: ClearColorConfig::None,
                            ..default()
                        },
                        ..default()
                    },
                    UiCameraConfig { show_ui: false },
                    RenderLayers::default().with(1),
                ));
            });
    });
}

fn apply_controls(keyboard: Res<Input<KeyCode>>, mut query: Query<&mut TnuaPlatformerControls>) {
    // if egui_context.ctx_mut().wants_keyboard_input() {
    //     for mut controls in query.iter_mut() {
    //         *controls = Default::default();
    //     }
    //     return;
    // }

    let mut direction = Vec3::ZERO;

    if keyboard.pressed(KeyCode::Up) {
        direction -= Vec3::Z;
    }
    if keyboard.pressed(KeyCode::Down) {
        direction += Vec3::Z;
    }
    if keyboard.pressed(KeyCode::Left) {
        direction -= Vec3::X;
    }
    if keyboard.pressed(KeyCode::Right) {
        direction += Vec3::X;
    }

    direction = direction.clamp_length_max(1.0);

    let jump = keyboard.pressed(KeyCode::Space);

    for mut controls in query.iter_mut() {
        *controls = TnuaPlatformerControls {
            desired_velocity: direction,
            desired_forward: direction.normalize(),
            jump: jump.then(|| 1.0),
        };
    }
}
