use bevy::{
    pbr::{CascadeShadowConfigBuilder, NotShadowCaster},
    prelude::*,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use cameras::flycam::FlycamPlugin;
use meshes::doom_light::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(FlycamPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(MaterialPlugin::<DoomLightMaterial>::default())
        .add_plugin(DoomLightsPlugin)
        .insert_resource(ClearColor(Color::DARK_GRAY))
        .add_startup_system(setup)
        .add_system(camera_orbit)
        .register_type::<DoomLight>()
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut doom_materials: ResMut<Assets<DoomLightMaterial>>,
) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(2., 1.5, 3.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        ..default()
    });

    commands.spawn((
        MaterialMeshBundle {
            mesh: meshes.add(Mesh::from(DoomLightMesh)),
            material: doom_materials.add(DoomLightMaterial {}),
            transform: Transform::from_rotation(Quat::from_rotation_x(-90.0_f32.to_radians()))
                .with_scale(Vec3::new(0.3, 1., 1.)),
            ..default()
        },
        DoomLight::default(),
        NotShadowCaster,
    ));
}

fn camera_orbit(time: Res<Time>, mut camera_q: Query<&mut Transform, With<Camera>>) {
    let Ok(mut tf) = camera_q.get_single_mut() else { return };
    tf.translate_around(Vec3::ZERO, Quat::from_rotation_y(time.elapsed_seconds().sin() * 0.05));
    tf.look_at(Vec3::ZERO, Vec3::Y);
}
