use bevy::{
    core_pipeline::clear_color::ClearColorConfig, pbr::NotShadowCaster, prelude::*, reflect::TypeUuid,
    render::render_resource::AsBindGroup,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use custom_pass::{
    core::{OcclusionPrepassLight, OcclusionPrepassOccluder, OcclusionViewPrepassTextures},
    OcclusionPrepassPlugin, PrepassPipelinePlugin, PrepassPlugin,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            watch_for_changes: true,
            ..Default::default()
        }))
        .add_plugin(OcclusionPrepassPlugin)
        .add_plugin(PrepassPipelinePlugin::<StandardMaterial>::default())
        .add_plugin(PrepassPlugin::<StandardMaterial>::default())
        .add_plugin(cameras::pan_orbit::PanOrbitCameraPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(MaterialPlugin::<PrepassOutputMaterial> {
            prepass_enabled: false,
            ..default()
        })
        .add_startup_system(setup)
        // .add_startup_system(setup_prepass_viewer)
        .add_system(cube_rotator)
        .add_system(wtf)
        .run();
}

#[derive(Component)]
struct MainCube;

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube::new(2.0))),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, -15.0)),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.8, 0.7, 0.6),
                reflectance: 0.2,
                ..default()
            }),
            ..default()
        },
        MainCube,
        OcclusionPrepassOccluder,
    ));

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(
                Mesh::try_from(shape::Icosphere {
                    radius: 6.0,
                    ..Default::default()
                })
                .unwrap(),
            ),
            material: materials.add(StandardMaterial {
                base_color: Color::WHITE,
                unlit: true,
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 0.0, -30.0),
            ..default()
        },
        NotShadowCaster,
        OcclusionPrepassLight,
    ));

    commands.insert_resource(AmbientLight {
        color: Color::ANTIQUE_WHITE,
        brightness: 0.4,
    });

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            camera_3d: Camera3d {
                clear_color: ClearColorConfig::Custom(Color::rgb(0.45, 0.76, 0.91)),
                ..default()
            },
            ..default()
        },
        custom_pass::core::OcclusionNormalPrepass,
        custom_pass::core::OcclusionDepthPrepass,
    ));
}

fn setup_prepass_viewer(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut depth_materials: ResMut<Assets<PrepassOutputMaterial>>,
) {
    commands.spawn((
        MaterialMeshBundle {
            mesh: meshes.add(shape::Quad::new(Vec2::new(20.0, 20.0)).into()),
            material: depth_materials.add(PrepassOutputMaterial { color_texture: None }),
            transform: Transform::from_xyz(0.0, 0.0, 0.0).looking_at(Vec3::new(0.0, 0.0, 5.0), Vec3::Y),
            ..default()
        },
        NotShadowCaster,
    ));
}

fn wtf(q: Query<&OcclusionViewPrepassTextures, With<Camera>>) {
    for texture in &q {
        println!("fuck me. depth tex: {:?}", texture.depth.is_some());
        println!("fuck me. depth tex: {:?}", texture.depth.is_some());
        println!("fuck me. depth tex: {:?}", texture.depth.is_some());
        println!("fuck me. depth tex: {:?}", texture.depth.is_some());
        println!("fuck me. depth tex: {:?}", texture.depth.is_some());
        println!("fuck me. depth tex: {:?}", texture.depth.is_some());
        println!("fuck me. depth tex: {:?}", texture.depth.is_some());
        println!("fuck me. depth tex: {:?}", texture.depth.is_some());
    }
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

#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "0af99895-b96e-4451-bc12-c6b1c1c52751"]
pub struct PrepassOutputMaterial {
    #[texture(0)]
    #[sampler(1)]
    color_texture: Option<Handle<Image>>,
}

impl Material for PrepassOutputMaterial {
    // This needs to be transparent in order to show the scene behind the mesh
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}
