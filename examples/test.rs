use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use cameras::flycam::FlycamPlugin;

#[derive(Component)]
struct ParticleSystem;

#[derive(Component)]
struct Particle;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(FlycamPlugin)
        .add_startup_system(setup)
        .add_startup_system(setup_particle_system)
        .add_system(tick_particles)
        .run();
}

/// set up a simple 3D scene
fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    // plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(5.0).into()),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });
    // cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
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
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn setup_particle_system(mut commands: Commands) {
    commands
        .spawn((ParticleSystem, Name::new("Parent")))
        .with_children(|parent| {
            for i in 0..5 {
                parent.spawn((Particle, Name::new(format!("Child {}", i))));
            }
        });
}

fn tick_particles(
    mut once: Local<bool>,
    mut commands: Commands,
    q_parent: Query<(Entity, &ParticleSystem, &Children)>,
    q_child: Query<(Entity, &Particle)>,
) {
    if *once {
        return;
    }
    *once = true;

    for (entity, _particle_system, children) in q_parent.iter() {
        commands.entity(entity).with_children(|parent| {
            parent.spawn((Particle, Name::new("New Baby")));
        });

        for (e, _particle) in q_child.iter_many(children) {
            println!("fook: {:?}", e);
        }
    }
}
