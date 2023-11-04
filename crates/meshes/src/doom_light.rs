// http://yzergame.com/doomGlare.html
// https://hollowdilnik.com/2022/06/20/doom-glow.html
use bevy::{
    math::Vec4Swizzles,
    prelude::*,
    reflect::{TypePath, TypeUuid},
    render::{
        mesh::{Indices, VertexAttributeValues},
        render_resource::{AsBindGroup, PrimitiveTopology, ShaderRef},
    },
};

pub struct DoomLightsPlugin;

impl Plugin for DoomLightsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_lights);
    }
}

fn update_lights(
    camera_q: Query<&Transform, With<Camera>>,
    mut light_q: Query<(&ViewVisibility, &Transform, &Handle<Mesh>, &mut DoomLight)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let Ok(cam_tf) = camera_q.get_single() else { return };

    for (visiblity, tf, mesh_handle, mut doom_light) in light_q.iter_mut() {
        if !visiblity.get() {
            continue;
        }
        let Some(mesh) = meshes.get_mut(mesh_handle) else {
            return;
        };

        // Everything is in local space unless said otherwise
        let tf_inverse_mat = tf.compute_matrix().inverse();
        let cam_pos =
            (tf_inverse_mat * Vec4::new(cam_tf.translation.x, cam_tf.translation.y, cam_tf.translation.z, 1.)).xyz();

        // is there any reason to not hardcode this?
        // let u = doom_light.verts[1] - doom_light.verts[0];
        // let v = doom_light.verts[2] - doom_light.verts[0];
        // let quad_normal = u.cross(v).normalize();
        let quad_normal = Vec3::new(0., 0., 1.);

        let ctr_pt: Vec3 =
            0.25 * (doom_light.verts[0] + doom_light.verts[1] + doom_light.verts[2] + doom_light.verts[3]);

        let dot = (ctr_pt - cam_pos).normalize().dot(quad_normal);
        let sign = dot.signum();

        if let Some(VertexAttributeValues::Float32x4(colors)) = mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR) {
            // Set colors from dot
            let alpha = map(dot.abs(), 0.001, 0.1, 0.0, 1.0);

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
                c[3] = 0.;
            }
        }

        // two-sided, do we need this or is turning off culling good enough?
        // if dot < 0. {
        //     doom_light.verts.swap(1, 3);
        //     sign = -sign;
        // }

        let eye_to_point_ws: Vec<_> = doom_light
            .verts
            .iter()
            .take(4)
            .map(|p| tf.transform_point(*p - cam_pos).normalize())
            .collect();

        // Extrude quad vertices
        let mut push_dir_ws = [Vec3::ZERO; 3];
        for i in 0..4 {
            push_dir_ws[0] = sign * (eye_to_point_ws[i].cross(eye_to_point_ws[(i + 3) % 4])).normalize();
            push_dir_ws[1] = sign * (eye_to_point_ws[(i + 1) % 4].cross(eye_to_point_ws[i])).normalize();
            push_dir_ws[2] = (push_dir_ws[0] + push_dir_ws[1]).normalize();

            for j in 0..3 {
                let mut offset = doom_light.push_distance * push_dir_ws[j];
                offset = (tf_inverse_mat * Vec4::new(offset.x, offset.y, offset.z, 1.)).xyz();
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
    let value = ((value - in_min) * (out_max - out_min)) / (in_max - in_min) + out_min;
    value.clamp(out_min, out_max)
}

#[derive(AsBindGroup, TypeUuid, Debug, Clone, TypePath, Asset)]
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f056e0"]
pub struct DoomLightMaterial {}

impl Material for DoomLightMaterial {
    fn fragment_shader() -> ShaderRef {
        "doom_light.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline<Self>,
        _descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        _layout: &bevy::render::mesh::MeshVertexBufferLayout,
        _key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        // to not cull and make the light two-sided
        // descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

// requires a MaterialMeshBundle with the DoomLightMesh: meshes.add(Mesh::from(DoomLightMesh))
// and DoomLightMaterial: doom_materials.add(DoomLightMaterial {}),
#[derive(Component, Reflect)]
pub struct DoomLight {
    push_distance: f32,
    quad_color: Color,
    edge_color: Color,
    verts: Vec<Vec3>,
}

impl Default for DoomLight {
    fn default() -> Self {
        let verts: Vec<_> = vec![
            Vec3::new(-1.0, -1.0, 0.0),
            Vec3::new(1.0, -1.0, 0.0),
            Vec3::new(1.0, 1.0, 0.0),
            Vec3::new(-1.0, 1.0, 0.0),
            Vec3::new(-1.0, -1.0, 0.0),
            Vec3::new(1.0, -1.0, 0.0),
            Vec3::new(1.0, 1.0, 0.0),
            Vec3::new(-1.0, 1.0, 0.0),
            Vec3::new(-1.0, -1.0, 0.0),
            Vec3::new(1.0, -1.0, 0.0),
            Vec3::new(1.0, 1.0, 0.0),
            Vec3::new(-1.0, 1.0, 0.0),
            Vec3::new(-1.0, -1.0, 0.0),
            Vec3::new(1.0, -1.0, 0.0),
            Vec3::new(1.0, 1.0, 0.0),
            Vec3::new(-1.0, 1.0, 0.0),
        ];

        Self {
            push_distance: 0.3,
            quad_color: Color::rgba(1., 1., 1., 1.),
            edge_color: Color::rgba(0., 1., 1., 0.),
            verts,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct DoomLightMesh;

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
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[1., 1., 1., 1.]; positions.len()]);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh
    }
}
