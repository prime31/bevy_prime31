use bevy::{
    prelude::{Mesh, Quat, Vec2, Vec3},
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};

use crate::{
    generate::{brush::BrushGeometry, brush_plane::PlaneGeometry, Geometry, Vertex},
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
        let mut positions: Vec<[f32; 3]> = Vec::new();
        let mut normals: Vec<[f32; 3]> = Vec::new();
        let mut uvs: Vec<[f32; 2]> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        for vert in mesh_surface.to_local().iter() {
            positions.push([vert.x, vert.y, vert.z]);
        }

        for normal in mesh_surface.normals.iter() {
            normals.push([normal.x, normal.y, normal.z]);
        }

        if let Some(node_uvs) = &mesh_surface.uvs {
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
        if uvs.len() > 0 {
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        }
        mesh.set_indices(Some(Indices::U32(indices)));

        mesh
    }
}

pub fn get_brush_entity_visual_geometry(entity_geometry: &Geometry) -> Vec<MeshSurface> {
    let textures: Vec<_> = entity_geometry
        .brush_geometry
        .iter()
        .flat_map(|brush| brush.plane_geometry.iter().map(|plane| plane.texture.clone()))
        .filter_map(|t| t)
        .collect();

    // Collect unique texture names
    let mut textures: Vec<Option<String>> = textures
        .clone()
        .into_iter()
        .enumerate()
        .filter(texture_filter::unique_not_blacklisted(&textures))
        .unzip::<usize, String, Vec<usize>, Vec<String>>()
        .1
        .into_iter()
        .map(Some)
        .collect();

    // Account for untextured brushes
    textures.push(None);

    // Build mesh surfaces for this texture
    let mesh_surfaces: Vec<MeshSurface> = textures
        .into_iter()
        .flat_map(build_brush_entity_texture_surface(entity_geometry))
        .collect();

    // Return mesh-type visual geometry
    mesh_surfaces
}

mod texture_filter {
    fn unique<'a>(textures: &'a [String]) -> impl Fn(&(usize, String)) -> bool + 'a {
        move |(i, texture): &(usize, String)| textures.iter().skip(i + 1).find(|comp| *comp == texture).is_none()
    }

    pub fn unique_not_blacklisted<'a>(textures: &'a [String]) -> impl Fn(&(usize, String)) -> bool + 'a {
        move |i: &(usize, String)| unique(textures)(i)
    }
}

fn build_brush_entity_texture_surface<'a>(
    entity_geometry: &'a Geometry,
) -> impl Fn(Option<String>) -> Option<MeshSurface> + 'a {
    move |texture| {
        let (vertices, indices) = gather_entity_geometry(entity_geometry, &texture);

        if vertices.is_empty() {
            return None;
        }

        let verts: Vec<Vec3> = vertices.iter().map(|vertex| vertex.vertex).collect();
        let normals: Vec<Vec3> = vertices.iter().map(|vertex| vertex.normal).collect();
        let tangents: Vec<(Vec3, f32)> = vertices.iter().map(|vertex| vertex.tangent).collect();
        let uvs: Option<Vec<Vec2>> = vertices.iter().map(|vertex| vertex.uv).collect();

        let mesh_surface = MeshSurface::new(texture, verts, normals, tangents, uvs, indices);
        Some(mesh_surface)
    }
}

fn gather_entity_geometry<'a>(
    entity_geometry: &'a Geometry,
    texture: &Option<String>,
) -> (Vec<&'a Vertex>, Vec<usize>) {
    let brush_geometry: Vec<(Vec<&Vertex>, Vec<usize>)> = entity_geometry
        .brush_geometry
        .iter()
        .map(|brush_geometry| gather_brush_geometry(brush_geometry, texture))
        .collect();

    let vertices: Vec<&Vertex> = brush_geometry
        .iter()
        .flat_map(|(vertices, _indices)| (*vertices).clone())
        .collect();

    let mut index_offset: usize = 0;
    let indices: Vec<usize> = brush_geometry
        .iter()
        .flat_map(|(vertices, indices)| {
            let indices = indices.clone().into_iter().map(move |index| index + index_offset);
            index_offset += vertices.len();
            indices
        })
        .collect();

    (vertices, indices)
}

fn gather_brush_geometry<'a>(
    brush_geometry: &'a BrushGeometry,
    texture: &Option<String>,
) -> (Vec<&'a Vertex>, Vec<usize>) {
    let plane_geometry = &brush_geometry.plane_geometry;

    let vertices: Vec<&Vertex> = plane_geometry
        .iter()
        .filter(|geo| geo.texture == *texture)
        .flat_map(move |plane_geometry| &plane_geometry.vertices)
        .collect();

    let mut index_offset: usize = 0;

    let concat_indices = |plane_geometry: &PlaneGeometry| {
        let indices = plane_geometry
            .indices
            .clone()
            .into_iter()
            .map(move |index| index + index_offset);

        index_offset += plane_geometry.vertices.len();

        indices
    };

    let indices: Vec<usize> = plane_geometry
        .iter()
        .filter(|geo| geo.texture == *texture)
        .flat_map(concat_indices)
        .collect();

    filter_vertices(&vertices, indices)
}

fn filter_vertices<'a>(vertices: &[&'a Vertex], indices: Vec<usize>) -> (Vec<&'a Vertex>, Vec<usize>) {
    let mut indices = indices;
    let mut new_indices: Vec<usize> = Vec::new();
    let mut new_vertices: Vec<&Vertex> = Vec::new();

    for (i, vertex) in vertices.iter().enumerate() {
        if unique(i, vertex, &vertices) {
            new_indices.push(i);
            new_vertices.push(vertex);
        } else {
            let position = vertices.iter().position(|comp| comp.vertex == vertex.vertex).unwrap();
            indices = indices
                .iter()
                .map(|index| if *index == i { position } else { *index })
                .collect();
        }
    }

    let indices: Vec<usize> = indices
        .iter()
        .flat_map(|index| new_indices.iter().position(|comp| comp == index))
        .collect();

    (new_vertices, indices)
}

fn unique(i: usize, vertex: &Vertex, vertices: &[&Vertex]) -> bool {
    let position = vertices.iter().position(|comp| {
        comp.vertex == vertex.vertex
            && comp.normal == vertex.normal
            && comp.tangent == vertex.tangent
            && comp.uv == vertex.uv
    });

    position.is_none() || position.unwrap() >= i
}

#[allow(dead_code)]
fn unique_position(i: usize, vertex: &Vertex, vertices: &[&Vertex]) -> bool {
    let position = vertices.iter().position(|comp| comp.vertex == vertex.vertex);
    position.is_none() || position.unwrap() >= i
}
