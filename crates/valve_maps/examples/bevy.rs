use std::time::Duration;

use bevy::{
    asset::ChangeWatcher,
    prelude::{
        default, AmbientLight, App, AssetPlugin, AssetServer, Camera3dBundle, Color, Commands, PluginGroup, Res,
        Transform, Vec3, *,
    },
    DefaultPlugins,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use cameras::flycam::FlycamPlugin;
use valve_maps::bevy::{ValveMapBundle, ValveMapPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            watch_for_changes: ChangeWatcher::with_delay(Duration::from_millis(200)),
            ..Default::default()
        }))
        .add_plugins(ValveMapPlugin)
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, setup_scene)
        .add_plugins(FlycamPlugin)
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.5,
        })
        .add_systems(Update, print_collision_events)
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
            // transform: Transform::from_xyz(-2.0, 6.5, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(valve_maps::bevy::ValveMapPlayer);

    commands.spawn(ValveMapBundle {
        map: asset_server.load("test.map"),
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
