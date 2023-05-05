use std::collections::HashMap;

use bevy::prelude::{Vec2, Vec3};

use crate::{
    convert::quake_point_to_bevy_point,
    formats::shared::{MapEntity, Plane},
};

pub fn entity_build(textures: &TextureInfo, entity: &MapEntity) -> Geometry {
    // Build brushes
    let brush_geometry: Vec<brush::BrushGeometry> = entity
        .brushes
        .iter()
        .map(|brush| brush::build(textures, entity, brush))
        .collect();

    Geometry::new(brush_geometry)
}

#[derive(Debug)]
pub struct TextureInfo(pub HashMap<String, TextureSize>);

impl TextureInfo {
    pub fn new() -> Self {
        TextureInfo(HashMap::new())
    }

    pub fn add_texture(&mut self, name: &str, width: u32, height: u32) -> &mut Self {
        self.0.insert(name.into(), TextureSize::new(width, height));
        self
    }
}

#[derive(Debug)]
pub struct TextureSize {
    pub width: u32,
    pub height: u32,
}

impl TextureSize {
    pub fn new(width: u32, height: u32) -> TextureSize {
        TextureSize { width, height }
    }

    pub fn size(&self) -> Vec2 {
        Vec2::new(self.width as f32, self.height as f32)
    }
}

#[derive(Debug, Clone)]
pub struct Geometry {
    pub brush_geometry: Vec<brush::BrushGeometry>,
}

impl Geometry {
    pub fn new(brush_geometry: Vec<brush::BrushGeometry>) -> Geometry {
        Geometry { brush_geometry }
    }

    pub fn get_convex_collision(&self) -> Vec<ConvexCollision> {
        self.brush_geometry
            .iter()
            .map(|brush_geo| {
                let points = brush_geo
                    .plane_geometry
                    .iter()
                    .flat_map(|brush_plane_geo| brush_plane_geo.vertices.iter().map(|vertex| vertex.vertex))
                    .collect::<Vec<Vec3>>();

                let points = points
                    .iter()
                    .enumerate()
                    .flat_map(|(i, vertex)| {
                        if points.iter().skip(i + 1).find(|comp| *comp == vertex).is_none() {
                            Some(*vertex)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<Vec3>>();

                ConvexCollision::new(points)
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct ConvexCollision {
    pub points: Vec<Vec3>,
}

impl ConvexCollision {
    pub fn new(points: Vec<Vec3>) -> ConvexCollision {
        ConvexCollision { points }
    }

    pub fn center(&self) -> Vec3 {
        self.points
            .iter()
            .fold(Vec3::new(0.0, 0.0, 0.0), |acc, next| acc + *next)
            / self.points.len().max(1) as f32
    }

    pub fn center_local(&self, inverse_scale_factor: f32) -> Vec3 {
        quake_point_to_bevy_point(self.center(), inverse_scale_factor)
    }

    /// this also converts from quake to bevy space
    pub fn to_local(&self, inverse_scale_factor: f32) -> Vec<Vec3> {
        let origin = self.center_local(inverse_scale_factor);

        self.points
            .iter()
            .fold(Vec::with_capacity(self.points.len()), |mut acc, next| {
                let vertex = quake_point_to_bevy_point(*next, inverse_scale_factor);
                let vertex = vertex - origin;
                acc.push(vertex);
                acc
            })
    }
}

pub mod brush {
    use crate::formats::shared::{Brush, MapEntity};
    use bevy::prelude::Vec3;

    use super::{brush_plane, TextureInfo};

    #[derive(Debug, Clone)]
    pub struct BrushGeometry {
        pub center: Vec3,
        pub plane_geometry: Vec<brush_plane::PlaneGeometry>,
    }

    impl<'a> BrushGeometry {
        pub fn new(center: Vec3, plane_geometry: Vec<brush_plane::PlaneGeometry>) -> BrushGeometry {
            BrushGeometry { center, plane_geometry }
        }
    }

    pub fn build(textures: &TextureInfo, entity: &MapEntity, brush: &Brush) -> BrushGeometry {
        let planes = &brush.planes;
        let plane_geometry: Vec<brush_plane::PlaneGeometry> = planes
            .iter()
            .map(|plane| brush_plane::build(textures, entity, planes, plane))
            .collect();

        // Calculate center
        let center = plane_geometry
            .iter()
            .fold(Vec3::new(0.0, 0.0, 0.0), |acc, plane_geometry| {
                acc + plane_geometry.center
            })
            / plane_geometry.len().max(1) as f32;

        BrushGeometry::new(center, plane_geometry)
    }
}

pub mod brush_plane {
    use std::cmp::Ordering;

    use bevy::prelude::Vec3;

    use crate::formats::shared::{MapEntity, Plane};

    use super::build_plane_vertex;
    use super::{TextureInfo, Vertex};

    #[derive(Debug, Clone)]
    pub struct PlaneGeometry {
        pub center: Vec3,
        pub vertices: Vec<Vertex>,
        pub indices: Vec<usize>,
        pub texture: Option<String>,
    }

    impl PlaneGeometry {
        pub fn new(center: Vec3, vertices: Vec<Vertex>, indices: Vec<usize>, texture: Option<String>) -> PlaneGeometry {
            PlaneGeometry {
                center,
                vertices,
                indices,
                texture,
            }
        }
    }

    pub fn build(
        TextureInfo(texture_info): &TextureInfo,
        entity: &MapEntity,
        planes: &[Plane],
        plane: &Plane,
    ) -> PlaneGeometry {
        let texture_info = texture_info.get(&plane.texture.name);

        let plane_vertices: Vec<Vertex> = planes
            .iter()
            .flat_map(|p1| {
                planes
                    .iter()
                    .flat_map(move |p2| build_plane_vertex(texture_info, entity, planes, plane, p1, p2))
            })
            .collect();

        let unique_vertices: Vec<Vertex> = plane_vertices
            .iter()
            .enumerate()
            .flat_map(|(i, vertex)| {
                // Find unique vertices and aggregate phong normals
                match plane_vertices
                    .iter()
                    .skip(i + 1)
                    .find(|comp| comp.vertex == vertex.vertex)
                {
                    None => match entity.fields.get_property("_phong") {
                        Some("1") => {
                            let mut vertex = plane_vertices.iter().skip(i + 1).fold(vertex.clone(), |mut acc, next| {
                                if next.vertex == acc.vertex {
                                    acc.normal = next.normal;
                                }
                                acc
                            });
                            vertex.normal = vertex.normal.normalize();
                            Some(vertex)
                        }
                        _ => Some(vertex.clone()),
                    },
                    _ => None,
                }
            })
            .collect();

        let center: Vec3 = unique_vertices
            .iter()
            .fold(Vec3::new(0.0, 0.0, 0.0), |acc, next| acc + next.vertex)
            / unique_vertices.len().max(1) as f32;

        let mut local_vertices = unique_vertices;
        for vertex in local_vertices.iter_mut() {
            vertex.vertex -= center;
        }

        let u_axis = (plane.v1() - plane.v0()).normalize();
        let v_axis = plane.normal().cross(u_axis);

        let mut wound_vertices = local_vertices;
        wound_vertices.sort_by(|a, b| {
            let vert_a = a.vertex;
            let vert_b = b.vertex;

            let lhs_pu = vert_a.dot(u_axis);
            let lhs_pv = vert_a.dot(v_axis);

            let rhs_pu = vert_b.dot(u_axis);
            let rhs_pv = vert_b.dot(v_axis);

            let lhs_angle = lhs_pv.atan2(lhs_pu);
            let rhs_angle = rhs_pv.atan2(rhs_pu);

            rhs_angle.partial_cmp(&lhs_angle).unwrap_or(Ordering::Equal)
        });

        let mut world_vertices = wound_vertices;
        for vertex in world_vertices.iter_mut() {
            vertex.vertex += center;
        }

        let mut indices: Vec<usize> = if world_vertices.len() < 3 {
            Vec::new()
        } else {
            (0..world_vertices.len() - 2)
                .flat_map(|i| vec![0, i + 1, i + 2])
                .collect()
        };
        indices.reverse();

        let texture = match texture_info {
            Some(_texture) => Some(plane.texture.name.clone()),
            None => None,
        };

        PlaneGeometry::new(center, world_vertices, indices, texture)
    }
}

#[derive(Debug, Clone)]
pub struct Vertex {
    pub vertex: Vec3,
    pub normal: Vec3,
    pub tangent: (Vec3, f32),
    pub uv: Option<Vec2>,
}

impl Vertex {
    pub fn new(vertex: Vec3, normal: Vec3, tangent: (Vec3, f32), uv: Option<Vec2>) -> Vertex {
        Vertex {
            vertex,
            normal,
            tangent,
            uv,
        }
    }
}

fn build_plane_vertex(
    texture_info: Option<&TextureSize>,
    entity: &MapEntity,
    planes: &[Plane],
    plane: &Plane,
    p1: &Plane,
    p2: &Plane,
) -> Option<Vertex> {
    if let Some(vertex) = intersect_brush_planes(plane, p1, p2) {
        if vertex_in_hull(vertex, planes) {
            let normal = vertex_normal(entity, plane, p1, p2);
            let tangent = valve_tangent(plane);

            let uv = match &texture_info {
                Some(texture) => Some(valve_uv(vertex, plane, texture)),
                None => None,
            };

            return Some(Vertex::new(vertex, normal, tangent, uv));
        }
    }

    None
}

fn valve_uv(vertex: Vec3, brush_plane: &Plane, texture: &TextureSize) -> Vec2 {
    let u_axis = brush_plane.texture.alignment.axes.u.normal.as_vec3();
    let v_axis = brush_plane.texture.alignment.axes.v.normal.as_vec3();

    let u_offset = brush_plane.texture.alignment.axes.u.offset;
    let v_offset = brush_plane.texture.alignment.axes.v.offset;

    let mut uv = Vec2::new(u_axis.dot(vertex), v_axis.dot(vertex));

    uv /= texture.size();
    uv /= Vec2::new(
        brush_plane.texture.alignment.scale.u,
        brush_plane.texture.alignment.scale.v,
    );
    uv += Vec2::new(u_offset, v_offset) / texture.size();

    uv
}

const CMP_EPSILON: f32 = 0.001;

pub fn intersect_brush_planes(p0: &Plane, p1: &Plane, p2: &Plane) -> Option<Vec3> {
    let n0 = p0.normal();
    let n1 = p1.normal();
    let n2 = p2.normal();

    let denom = n0.cross(n1).dot(n2);

    if denom < CMP_EPSILON {
        return None;
    }

    Some((n1.cross(n2) * p0.dist() + n2.cross(n0) * p1.dist() + n0.cross(n1) * p2.dist()) / denom)
}

pub fn vertex_in_hull(vertex: Vec3, hull: &[Plane]) -> bool {
    for brush_plane in hull {
        let proj = brush_plane.normal().dot(vertex);
        if proj > brush_plane.dist() && proj - brush_plane.dist() > CMP_EPSILON {
            return false;
        }
    }
    true
}

const ONE_DEGREE: f32 = 0.017_453_3;

pub fn vertex_normal(entity: &MapEntity, p0: &Plane, p1: &Plane, p2: &Plane) -> Vec3 {
    if let Some("1") = entity.fields.get_property("_phong") {
        return phong_normal(p0, p1, p2, entity.fields.get_property("_phong_angle"));
    }

    p0.normal()
}

fn phong_normal(p0: &Plane, p1: &Plane, p2: &Plane, phong_angle: Option<&str>) -> Vec3 {
    if let Some(phong_angle) = phong_angle {
        if let Ok(phong_angle) = phong_angle.parse::<f32>() {
            let threshold = ((phong_angle + 0.01) * ONE_DEGREE).cos();
            let mut normal = p0.normal();
            if p0.normal().dot(p1.normal()) > threshold {
                normal += p1.normal()
            }
            if p0.normal().dot(p2.normal()) > threshold {
                normal += p2.normal()
            }
            return normal.normalize();
        }
    }

    (p0.normal() + p1.normal() + p2.normal()).normalize()
}

fn valve_tangent(brush_plane: &Plane) -> (Vec3, f32) {
    let u_axis = brush_plane.texture.alignment.axes.u.normal.as_vec3();
    let v_axis = brush_plane.texture.alignment.axes.v.normal.as_vec3();

    let v_sign = -brush_plane.normal().cross(u_axis).dot(v_axis).signum();
    (u_axis, v_sign)
}
