use bevy::{
    pbr::CascadeShadowConfigBuilder,
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::{Indices, VertexAttributeValues},
        render_resource::{AsBindGroup, PrimitiveTopology, ShaderRef},
    },
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use cameras::flycam::FlycamPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(FlycamPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(MaterialPlugin::<DoomLightMaterial>::default())
        .add_plugin(DoomLightsPlugin)
        .add_plugin(bevy::pbr::wireframe::WireframePlugin)
        .add_startup_system(setup)
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

    let mesh = meshes.add(Mesh::from(DoomLightMesh));

    commands.spawn((
        MaterialMeshBundle {
            mesh: mesh.clone(),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            material: doom_materials.add(DoomLightMaterial {}),
            ..default()
        },
        DoomLight::new(mesh),
        // bevy::pbr::wireframe::Wireframe,
    ));
}

pub struct DoomLightsPlugin;

impl Plugin for DoomLightsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update_lights);
    }
}

fn update_lights(
    camera_q: Query<&Transform, With<Camera>>,
    mut light_q: Query<(&ComputedVisibility, &Transform, &mut DoomLight)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let Ok(cam_tf) = camera_q.get_single() else { return };

    for (visiblity, tf, mut doom_light) in light_q.iter_mut() {
        if !visiblity.is_visible() {
            continue;
        }
        let Some(mesh) = meshes.get_mut(&doom_light.mesh) else { return };

        let dir_to_center = (tf.translation - cam_tf.translation).normalize();
        let quad_normal = Vec3::new(0., 0., 1.);
        let dot = dir_to_center.dot(quad_normal);

        if let Some(VertexAttributeValues::Float32x4(colors)) = mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR) {
            // Set colors from dot
            let alpha = map(dot.abs(), 0.001, 0.1, 0.0, 1.0).clamp(0., 1.);

            // quad
            for c in colors.iter_mut().take(4) {
                c[0] = doom_light.quad_color.r();
                c[1] = doom_light.quad_color.g();
                c[2] = doom_light.quad_color.b();
                c[3] = doom_light.quad_color.a() * alpha;
            }

            // Flaps and connections
            for c in colors.iter_mut().skip(4) {
                c[0] = doom_light.edge_color.r();
                c[1] = doom_light.edge_color.g();
                c[2] = doom_light.edge_color.b();
                c[3] = doom_light.edge_color.a() * alpha;
            }
        }

        // Get worldspace eye to original 4 vertices
        let cam_local_pos = cam_tf.translation - tf.translation;
        //eyeToVerticesWorldSpace[index] = this.vertices[index].clone().sub(cameraLocalPosition).normalize()
        let eye_to_vert_ws: Vec<_> = doom_light
            .verts
            .iter()
            .take(4)
            .map(|pos| (cam_local_pos - *pos).normalize())
            .collect();

        // Extrude quad vertices
        let sign = dot.signum();
        let mut push_dir_ws = [Vec3::ZERO; 3];
        for i in 0..4 {
            //pushDirectionsWorldSpace[0] = eyeToVerticesWorldSpace[i].cross(eyeToVerticesWorldSpace[(i + 3) % 4]).scale(sign).normalize();
            //pushDirectionsWorldSpace[1] = eyeToVerticesWorldSpace[(i + 1) % 4].cross(eyeToVerticesWorldSpace[i]).scale(sign).normalize();
            //pushDirectionsWorldSpace[2] = pushDirectionsWorldSpace[0].add(pushDirectionsWorldSpace[1]).normalize();
            push_dir_ws[0] = (eye_to_vert_ws[i].cross(eye_to_vert_ws[(i + 3) % 4]) * sign).normalize();
            push_dir_ws[1] = (eye_to_vert_ws[(i + 1) % 4].cross(eye_to_vert_ws[i]) * sign).normalize();
            push_dir_ws[2] = (push_dir_ws[0] + push_dir_ws[1]).normalize();

            let push_distance = 0.5;
            for j in 0..3 {
                //const offset = pushDirectionsWorldSpace[j].clone().scale(pushDistance);
                //this.vertices[4 + j + 3 * i] = this.vertices[i].clone().add(offset);
                let offset = push_dir_ws[j] * push_distance;
                doom_light.verts[4 + j + 3 * i] = doom_light.verts[i] + offset;
            }
        }

        // set mesh positions from verts
        if let Some(VertexAttributeValues::Float32x3(positions)) = mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION) {
            for i in 0..positions.len() {
                positions[i] = doom_light.verts[i].into();
            }
        }
    }
}

// Value mapped to the range [out_min, out_max]
fn map(value: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    ((value - in_min) * (out_max - out_min)) / (in_max - in_min) + out_min
}

impl Material for DoomLightMaterial {
    fn fragment_shader() -> ShaderRef {
        "doom_light.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline<Self>,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        _layout: &bevy::render::mesh::MeshVertexBufferLayout,
        _key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f056e0"]
pub struct DoomLightMaterial {}

#[derive(Component)]
struct DoomLight {
    mesh: Handle<Mesh>,
    quad_color: Color,
    edge_color: Color,
    verts: Vec<Vec3>,
}

impl DoomLight {
    fn new(mesh: Handle<Mesh>) -> Self {
        let verts: Vec<_> = vec![
            [-1.0, -1.0, 0.0],
            [1.0, -1.0, 0.0],
            [1.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0],
            [-1.0, -1.0, 0.0],
            [1.0, -1.0, 0.0],
            [1.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0],
            [-1.0, -1.0, 0.0],
            [1.0, -1.0, 0.0],
            [1.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0],
            [-1.0, -1.0, 0.0],
            [1.0, -1.0, 0.0],
            [1.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0],
        ]
        .iter()
        .map(|p| Vec3::from(*p))
        .collect();

        Self {
            mesh,
            quad_color: Color::rgba(1., 1., 1., 0.95),
            edge_color: Color::rgba(0., 0., 0., 0.),
            verts,
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct DoomLightMesh;

impl From<DoomLightMesh> for Mesh {
    fn from(_doom_light: DoomLightMesh) -> Self {
        let positions = vec![
            [-1.0, -1.0, 0.0],
            [1.0, -1.0, 0.0],
            [1.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0],
            [-1.0, -1.0, 0.0],
            [1.0, -1.0, 0.0],
            [1.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0],
            [-1.0, -1.0, 0.0],
            [1.0, -1.0, 0.0],
            [1.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0],
            [-1.0, -1.0, 0.0],
            [1.0, -1.0, 0.0],
            [1.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0],
        ];

        let indices = Indices::U32(vec![
            0, 1, 2, 0, 2, 3, // quad
            0, 5, 7, 0, 7, 1, 1, 8, 10, 1, 10, 2, 2, 11, 13, 2, 13, 3, 3, 14, 4, 3, 4, 0, // Flaps
            0, 4, 6, 0, 6, 5, 1, 7, 9, 1, 9, 8, 2, 10, 12, 2, 12, 11, 3, 13, 15, 3, 15, 14, // Connections
        ]);

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(indices));
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[1., 0.1, 1., 1.]; positions.len()]);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh
    }
}
