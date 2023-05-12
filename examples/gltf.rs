use bevy::{
    gltf::{Gltf, GltfMesh},
    pbr::CascadeShadowConfigBuilder,
    prelude::*,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use cameras::flycam::FlycamPlugin;

#[derive(Resource, Default)]
struct GltfState {
    is_loaded: bool,
    handle: Handle<Gltf>,
}

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0 / 5.0f32,
        })
        .insert_resource(GltfState::default())
        .add_plugins(DefaultPlugins)
        .add_plugin(FlycamPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .add_systems((setup.on_startup(), check_loaded_scene))
        .run();
}

fn setup(mut commands: Commands, mut gltf_state: ResMut<GltfState>, asset_server: Res<AssetServer>) {
    gltf_state.handle = asset_server.load(String::from("models/monkey.gltf"));

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(2., 1.5, 3.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        ..default()
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        cascade_shadow_config: CascadeShadowConfigBuilder {
            num_cascades: 1,
            maximum_distance: 1.6,
            ..default()
        }
        .into(),
        ..default()
    });
}

fn check_loaded_scene(
    mut commands: Commands,
    gltf_assets: Res<Assets<Gltf>>,
    mut gltf_state: ResMut<GltfState>,
    scenes: Res<Assets<Scene>>,
    meshes: Res<Assets<GltfMesh>>,
) {
    if gltf_state.is_loaded {
        return;
    };

    if let Some(gltf) = gltf_assets.get(&gltf_state.handle) {
        for scene in &gltf.scenes {
            if let Some(scene) = scenes.get(scene) {
                println!("scene: {:?}", scene);
            }
        }

        for mesh in &gltf.meshes {
            if let Some(mesh) = meshes.get(mesh) {
                println!("mesh: {:?}", mesh);
            }
        }

        commands.spawn(SceneBundle {
            scene: gltf.scenes[0].clone(),
            ..default()
        });

        gltf_state.is_loaded = true;
    }
}
