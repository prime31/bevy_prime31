use std::f32::consts::TAU;

use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    math::Vec3Swizzles,
    prelude::{
        default, AmbientLight, App, AssetPlugin, AssetServer, Camera3dBundle, Color, Commands, PluginGroup, Res,
        Transform, Vec3, *,
    },
    render::{camera::Viewport, view::RenderLayers},
    DefaultPlugins,
};

use bevy_rapier3d::prelude::*;

use egui_helper::EguiHelperPlugin;
use fps_controller::{
    character_controller::CharacterControllerPlugin,
    input::{FpsInputPlugin, FpsPlayer, RenderPlayer},
};
use valve_maps::bevy::{ValveMapBundle, ValveMapPlayer, ValveMapPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            watch_for_changes: true,
            ..Default::default()
        }))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.5,
        })
        .add_plugin(EguiHelperPlugin)
        .add_plugin(ValveMapPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(FpsInputPlugin)
        .add_plugin(CharacterControllerPlugin)
        .add_startup_system(setup_scene)
        .add_systems((print_collision_events, display_text))
        .run();
}

fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    assets: Res<AssetServer>,
) {
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::rgb(0.98, 0.95, 0.82),
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 0.0, 0.0).looking_at(Vec3::new(-0.15, -0.05, 0.25), Vec3::Y),
        ..default()
    });

    commands
        .spawn(Collider::cuboid(40.0, 0.1, 40.0))
        .insert(Restitution::coefficient(1.0))
        .insert(TransformBundle::from(Transform::from_xyz(0.0, -2.0, 0.0)));

    // FPS player with a child camera with has another child camera (for 3rd person view)
    commands
        .spawn((
            (ValveMapPlayer, FpsPlayer, RenderLayers::layer(1)),
            PbrBundle {
                mesh: meshes.add(shape::Capsule::default().into()),
                material: materials.add(Color::rgb(0.8, 0.1, 0.9).into()),
                ..Default::default()
            },
            Collider::capsule(Vec3::Y * -0.5, Vec3::Y * 0.5, 0.5),
            Velocity::zero(),
            ActiveEvents::COLLISION_EVENTS,
            RigidBody::KinematicPositionBased,
            KinematicCharacterController::default(),
            KinematicCharacterControllerOutput::default(),
            Ccd { enabled: true }, // Prevent clipping when going fast
        ))
        .with_children(|builder| {
            builder
                .spawn((
                    RenderPlayer,
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

    commands.spawn(
        TextBundle::from_section(
            "",
            TextStyle {
                font: assets.load("fira_mono.ttf"),
                font_size: 24.0,
                color: Color::BLACK,
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                ..default()
            },
            ..default()
        }),
    );

    commands.spawn(ValveMapBundle {
        map: asset_server.load("playground.map"),
        ..Default::default()
    });
}

fn print_collision_events(
    mut collision_events: EventReader<CollisionEvent>,
    mut contact_force_events: EventReader<ContactForceEvent>,
) {
    for collision_event in collision_events.iter() {
        println!("Received collision event: {:?}", collision_event);
    }

    for contact_force_event in contact_force_events.iter() {
        println!("Received contact force event: {:?}", contact_force_event);
    }
}

fn display_text(mut controller_query: Query<(&Transform, &Velocity)>, mut text_query: Query<&mut Text>) {
    for (transform, velocity) in &mut controller_query {
        for mut text in &mut text_query {
            text.sections[0].value = format!(
                "vel: {:.2}, {:.2}, {:.2}\npos: {:.2}, {:.2}, {:.2}\nspd: {:.2}",
                velocity.linvel.x,
                velocity.linvel.y,
                velocity.linvel.z,
                transform.translation.x,
                transform.translation.y,
                transform.translation.z,
                velocity.linvel.xz().length()
            );
        }
    }
}
