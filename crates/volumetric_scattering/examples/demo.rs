use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    pbr::NotShadowCaster,
    prelude::*,
    reflect::TypeUuid,
    render::{
        camera::RenderTarget,
        render_resource::{
            AsBindGroup, Extent3d, ShaderRef, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        view::RenderLayers,
    },
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle},
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            watch_for_changes: true,
            ..Default::default()
        }))
        .add_plugin(Material2dPlugin::<VolumetricScatteringMaterial>::default())
        .add_plugin(cameras::pan_orbit::PanOrbitCameraPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .add_startup_system(setup)
        .add_system(cube_rotator)
        .run();
}

#[derive(Component)]
struct MainCube;

fn setup(
    mut commands: Commands,
    windows: Query<&Window>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut volumetric_scattering_materials: ResMut<Assets<VolumetricScatteringMaterial>>,
) {
    let window = windows.single();

    let size = Extent3d {
        width: window.resolution.physical_width(),
        height: window.resolution.physical_height(),
        ..default()
    };

    let mut main_image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    // fill image.data with zeroes
    main_image.resize(size);
    let main_image_handle = images.add(main_image);

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
        RenderLayers::layer(0),
    ));

    commands.insert_resource(AmbientLight {
        color: Color::ANTIQUE_WHITE,
        brightness: 0.4,
    });

    commands.spawn(PointLightBundle { ..default() });

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
            camera: Camera {
                target: RenderTarget::Image(main_image_handle.clone()),
                ..default()
            },
            ..default()
        },
        RenderLayers::layer(0),
    ));

    let window = windows.single();

    let size = Extent3d {
        width: window.resolution.physical_width(),
        height: window.resolution.physical_height(),
        ..default()
    };

    let mut occlusion_image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    // fill image.data with zeroes
    occlusion_image.resize(size);
    let occlusion_image_handle = images.add(occlusion_image);

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube::new(2.0))),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
            material: materials.add(StandardMaterial {
                base_color: Color::BLACK,
                unlit: true,
                ..default()
            }),
            ..default()
        },
        MainCube,
        NotShadowCaster,
        RenderLayers::layer(1),
    ));

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(
                Mesh::try_from(shape::Icosphere {
                    radius: 4.0,
                    ..Default::default()
                })
                .unwrap(),
            ),
            material: materials.add(StandardMaterial {
                base_color: Color::WHITE,
                unlit: true,
                ..default()
            }),
            ..default()
        },
        NotShadowCaster,
        RenderLayers::layer(1),
    ));

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 25.0)).looking_at(Vec3::default(), Vec3::Y),
            camera_3d: Camera3d {
                clear_color: ClearColorConfig::Custom(Color::BLACK),
                ..default()
            },
            camera: Camera {
                order: 1,
                target: RenderTarget::Image(occlusion_image_handle.clone()),
                ..default()
            },
            ..default()
        },
        RenderLayers::layer(1),
    ));

    // setup a quad for the final render
    let quad_handle = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(
        size.width as f32,
        size.height as f32,
    ))));

    // This material has the texture that has been rendered.
    let material_handle = volumetric_scattering_materials.add(VolumetricScatteringMaterial {
        source_image: main_image_handle,
        occlusion_image: occlusion_image_handle,
    });

    // Post processing 2d quad, with material using the render texture done by the main camera, with a custom shader.
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: quad_handle.into(),
            material: material_handle,
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 1.5),
                ..default()
            },
            ..default()
        },
        RenderLayers::layer(2),
    ));

    // The post-processing pass camera.
    commands.spawn((
        Camera2dBundle {
            camera: Camera { order: 2, ..default() },
            ..Camera2dBundle::default()
        },
        RenderLayers::layer(2),
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

#[derive(AsBindGroup, TypeUuid, Clone)]
#[uuid = "bc2f08ec-a0fb-43f1-a908-54871ea597d5"]
struct VolumetricScatteringMaterial {
    #[texture(0)]
    #[sampler(1)]
    source_image: Handle<Image>,
    #[texture(2)]
    #[sampler(3)]
    occlusion_image: Handle<Image>,
}

impl Material2d for VolumetricScatteringMaterial {
    fn fragment_shader() -> ShaderRef {
        "volumetric_scattering.wgsl".into()
    }
}
