use bevy::{DefaultPlugins, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use cameras::flycam::FlycamPlugin;
use meshes::SphericalHelix;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(FlycamPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(2., 1.5, 3.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        ..default()
    });

    // create the Mesh
    let helix_mesh = Mesh::from(SphericalHelix::new(50, 2.5, 2.0, 0.7));

    // stick it on an entity with a material
    commands.spawn(PbrBundle {
        mesh: meshes.add(helix_mesh),
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.3, 0.3, 0.6),
            cull_mode: None,
            double_sided: true,
            unlit: true,
            ..Default::default()
        }),
        transform: Transform::from_xyz(0.0, 2.0, 0.0),
        ..default()
    });
}
