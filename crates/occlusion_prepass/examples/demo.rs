use bevy::{core_pipeline::clear_color::ClearColorConfig, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use occlusion_prepass::{PrepassPipelinePlugin, PrepassPlugin, OcclusionPrepass};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            watch_for_changes: true,
            ..Default::default()
        }))
        .add_plugin(PrepassPipelinePlugin::<StandardMaterial>::default())
        .add_plugin(PrepassPlugin::<StandardMaterial>::default())
        .add_plugin(cameras::pan_orbit::PanOrbitCameraPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .add_startup_system(setup)
        .add_system(cube_rotator)
        .run();
}

#[derive(Component)]
struct MainCube;

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube::new(2.0))),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.8, 0.7, 0.6),
                reflectance: 0.2,
                ..default()
            }),
            ..default()
        },
        MainCube,
    ));

    commands.insert_resource(AmbientLight {
        color: Color::ANTIQUE_WHITE,
        brightness: 0.4,
    });

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1800.0,
            range: 20.0,
            ..Default::default()
        },
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 5.0)),
        ..Default::default()
    });

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 25.0)).looking_at(Vec3::default(), Vec3::Y),
            camera_3d: Camera3d {
                clear_color: ClearColorConfig::Custom(Color::rgb(0.45, 0.76, 0.91)),
                ..default()
            },
            ..default()
        },
        bevy::core_pipeline::prepass::DepthPrepass,
        OcclusionPrepass,
    ));
}

fn cube_rotator(time: Res<Time>, mut query: Query<&mut Transform, With<MainCube>>, mut angle: Local<f32>) {
    for mut transform in &mut query {
        transform.rotate_x(0.55 * time.delta_seconds());
        transform.rotate_z(0.15 * time.delta_seconds());

        /// maps value (which is in the range left_min - left_max) to a value in the range right_min - right_max
        pub fn map(value: f32, left_min: f32, left_max: f32, right_min: f32, right_max: f32) -> f32 {
            let slope = (right_max - right_min) / (left_max - left_min);
            right_min + slope * (value - left_min)
        }

        // transform.translate_around(Vec3::ZERO, Quat::from_rotation_y(angle.to_radians()));
        transform.scale = Vec3::splat(map(f32::sin(time.elapsed_seconds()), -1.0, 1.0, 1.0, 2.0));

        *angle += 0.1 * time.delta_seconds();
        if *angle > 360.0 {
            *angle = 0.0;
        }
    }
}
