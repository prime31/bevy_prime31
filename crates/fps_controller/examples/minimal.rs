use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    prelude::{
        default, AmbientLight, App, AssetPlugin, AssetServer, Camera3dBundle, Color, Commands, PluginGroup, Res,
        Transform, Vec3, *,
    },
    render::camera::Viewport,
    DefaultPlugins,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use cameras::flycam::FlycamPlugin;
use fps_controller::FPSControllerPlugin;
use valve_maps::bevy::{ValveMapBundle, ValveMapPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            watch_for_changes: true,
            ..Default::default()
        }))
        .add_plugin(ValveMapPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(FPSControllerPlugin)
        .add_startup_system(setup_scene)
        .add_plugin(FlycamPlugin)
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.5,
        })
        .add_system(print_collision_events)
        .run();
}

fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn(Collider::cuboid(10.0, 0.1, 10.0))
        .insert(Restitution::coefficient(1.0))
        .insert(TransformBundle::from(Transform::from_xyz(0.0, -2.0, 0.0)));
    commands
        .spawn(RigidBody::Dynamic)
        .insert(Collider::ball(1.0))
        .insert(Restitution::coefficient(1.0))
        .insert(GravityScale(0.5))
        .insert(Velocity {
            linvel: Vec3::new(0.0, 10.0, 0.0),
            angvel: Vec3::new(0.2, 0.0, 0.0),
        })
        .insert(PbrBundle {
            mesh: meshes.add(shape::UVSphere::default().into()),
            material: materials.add(Color::rgb(0.8, 0.1, 0.9).into()),
            transform: Transform::from_xyz(0.0, 4.0, 0.0),
            ..Default::default()
        });

    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 6.5, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(valve_maps::bevy::ValveMapPlayer)
        .with_children(|builder| {
            // Right Camera
            let win_w = 1280;
            let frame_w = 256;
            let frame_h = 256 / (1280 / 720);
            println!("---- {}", frame_h);
            builder.spawn(Camera3dBundle {
                transform: Transform::from_xyz(0., 1.5, 6.).looking_at(Vec3::ZERO, Vec3::Y),
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
            });
        });

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
