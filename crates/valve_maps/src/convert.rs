use bevy::{
    prelude::{Mesh, Quat, Vec2, Vec3},
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};

pub fn quake_point_to_bevy_point(point: Vec3, inverse_scale_factor: f32) -> Vec3 {
    quake_direction_to_bevy_direction(point) / inverse_scale_factor
}

pub fn quake_direction_to_bevy_direction(dir: Vec3) -> Vec3 {
    let rot = Quat::from_axis_angle(Vec3::new(-1.0, 0.0, 0.0), 90.0_f32.to_radians());
    rot * dir
}

#[derive(Debug)]
pub struct MeshSurface {
    pub texture: Option<String>,
    pub vertices: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub tangents: Vec<(Vec3, f32)>,
    pub uvs: Option<Vec<Vec2>>,
    pub indices: Vec<usize>,
}

impl MeshSurface {
    pub fn new(
        texture: Option<String>,
        vertices: Vec<Vec3>,
        normals: Vec<Vec3>,
        tangents: Vec<(Vec3, f32)>,
        uvs: Option<Vec<Vec2>>,
        indices: Vec<usize>,
    ) -> MeshSurface {
        MeshSurface {
            texture,
            vertices,
            normals,
            tangents,
            uvs,
            indices,
        }
    }

    pub fn center(&self) -> Vec3 {
        self.vertices
            .iter()
            .fold(Vec3::new(0.0, 0.0, 0.0), |acc, next| acc + *next)
            / self.vertices.len().max(1) as f32
    }

    /// this also converts from quake to bevy space
    pub fn to_local(&self) -> Vec<Vec3> {
        let origin = self.center();

        self.vertices
            .iter()
            .fold(Vec::with_capacity(self.vertices.len()), |mut acc, next| {
                let vertex = *next - origin;
                acc.push(vertex);
                acc
            })
    }
}

impl From<&MeshSurface> for Mesh {
    fn from(mesh_surface: &MeshSurface) -> Mesh {
        let mut positions: Vec<[f32; 3]> = Vec::with_capacity(mesh_surface.vertices.len());
        let mut normals: Vec<[f32; 3]> = Vec::with_capacity(mesh_surface.normals.len());
        let mut uvs: Vec<[f32; 2]> = Vec::new();
        let mut indices: Vec<u32> = Vec::with_capacity(mesh_surface.indices.len());

        for vert in mesh_surface.to_local().iter() {
            positions.push([vert.x, vert.y, vert.z]);
        }

        for normal in mesh_surface.normals.iter() {
            normals.push([normal.x, normal.y, normal.z]);
        }

        if let Some(node_uvs) = &mesh_surface.uvs {
            uvs.reserve(node_uvs.len());
            for uv in node_uvs.iter() {
                uvs.push([uv.x, uv.y]);
            }
        }

        for i in mesh_surface.indices.iter() {
            indices.push(*i as u32);
        }

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.set_indices(Some(Indices::U32(indices)));
        if uvs.len() > 0 {
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
            if let Err(e) = mesh.generate_tangents() {
                println!("error generating tangents: {:?}", e);
            }
        }

        mesh
    }
}
