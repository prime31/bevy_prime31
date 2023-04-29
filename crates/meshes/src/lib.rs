use std::f32::consts::PI;

use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};

pub mod doom_light;

#[derive(Debug, Clone)]
pub struct SphericalHelix {
    pub steps: usize,
    pub twist: f32,
    pub radius: f32,
    pub width: f32,
}

impl SphericalHelix {
    pub fn new(steps: usize, twist: f32, radius: f32, width: f32) -> Self {
        return SphericalHelix {
            steps,
            twist,
            radius,
            width,
        };
    }
}

impl Default for SphericalHelix {
    fn default() -> Self {
        return SphericalHelix {
            steps: 50,
            twist: 2.5,
            radius: 2.0,
            width: 0.7,
        };
    }
}

impl From<SphericalHelix> for Mesh {
    fn from(helix: SphericalHelix) -> Self {
        struct VertexPosNormTanUv {
            positions: Vec<[f32; 3]>,
            normals: Vec<[f32; 3]>,
            tangents: Vec<[f32; 4]>,
            uvs: Vec<[f32; 2]>,
        }

        impl VertexPosNormTanUv {
            fn with_capacity(capacity: usize) -> VertexPosNormTanUv {
                VertexPosNormTanUv {
                    positions: vec![[0.0, 0.0, 0.0]; capacity],
                    normals: vec![[0.0, 0.0, 0.0]; capacity],
                    tangents: vec![[0.0, 0.0, 0.0, 0.0]; capacity],
                    uvs: vec![[0.0, 0.0]; capacity],
                }
            }
        }

        let mut spiral_pts = vec![Vec3::ZERO; helix.steps];
        let mut normals = vec![Vec3::ZERO; helix.steps];
        let mut tangents = vec![Vec3::ZERO; helix.steps];

        let a_max = helix.twist * std::f32::consts::PI * 2.0;
        for i in 0..helix.steps {
            let a = i as f32 / (helix.steps as f32 - 1.0) * a_max;
            let a_div_amax_pi = std::f32::consts::PI * (a / a_max);
            let x = helix.radius * a.cos() * (-std::f32::consts::PI / 2.0 + a_div_amax_pi).cos();
            let y = helix.radius * a.sin() * (-std::f32::consts::PI / 2.0 + a_div_amax_pi).cos();
            let z = helix.radius * (-std::f32::consts::PI / 2.0 + a_div_amax_pi).sin();

            spiral_pts[i] = Vec3 { x: x, y: y, z: z };
        }

        // tangents and normals
        for i in 0..helix.steps {
            // edge
            normals[i] = spiral_pts[i].normalize();

            let binormal = {
                if i < spiral_pts.len() - 1 {
                    spiral_pts[i + 1] - spiral_pts[i]
                } else {
                    tangents[i - 1]
                }
            }
            .normalize();

            tangents[i] = Vec3::cross(binormal, normals[i]);
        }

        // smooth, skip end points
        for i in 0..helix.steps - 1 {
            // edge
            normals[i] = (normals[i] + normals[i + 1]).normalize();
            tangents[i] = (tangents[i] + tangents[i + 1]).normalize();
        }

        let max = spiral_pts.len() - 1;

        // Finaly assemble vertices
        let mut verts = VertexPosNormTanUv::with_capacity(helix.steps * 2);
        for i in 0..spiral_pts.len() {
            let phase = i as f32 / max as f32;

            verts.positions[i * 2] = (spiral_pts[i] + tangents[i] * helix.width * 0.5).to_array();
            verts.normals[i * 2] = normals[i].to_array();
            verts.tangents[i * 2] = [tangents[i][0], tangents[i][1], tangents[i][2], 0.0];
            verts.uvs[i * 2] = [phase, 0.0];

            // spiral_pts[i] += Vec3::new(0.0, 0.0, 0.5);
            verts.positions[i * 2 + 1] = (spiral_pts[i] - tangents[i] * helix.width * 0.5).to_array();
            verts.normals[i * 2 + 1] = normals[i].to_array();
            verts.tangents[i * 2 + 1] = [-tangents[i][0], -tangents[i][1], -tangents[i][2], 0.0];
            verts.uvs[i * 2 + 1] = [phase, 1.0];
        }

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleStrip);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, verts.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, verts.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_TANGENT, verts.tangents);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, verts.uvs);

        mesh
    }
}

#[derive(Debug, Clone)]
pub struct Ring {
    pub sides: usize,
    pub width: f32,
}

impl Ring {
    pub fn new(sides: usize, width: f32) -> Self {
        return Ring {
            sides: sides,
            width: width,
        };
    }
}

impl Default for Ring {
    fn default() -> Self {
        return Ring { sides: 10, width: 0.5 };
    }
}

impl From<Ring> for Mesh {
    fn from(ring: Ring) -> Mesh {
        let mut positions: Vec<[f32; 3]> = Vec::with_capacity(ring.sides * 2 + 2);
        let mut normals: Vec<[f32; 3]> = Vec::with_capacity(ring.sides * 2 + 2);
        let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(ring.sides * 2 + 2);

        let angular_step = 2.0 * PI / ring.sides as f32;

        // closed strip
        for i in 0..=ring.sides {
            let mut angle = angular_step * i as f32;

            // just being safe here, or just copy first two vertices, I don't care
            if i == ring.sides {
                angle = 0.0;
            }

            // to make Pizza happy
            let (x, y) = angle.sin_cos();
            let normal = Vec3::new(x, y, 0.0);
            let position = normal;

            let phase = i as f32 / ring.sides as f32;
            positions.push((position - normal * ring.width * 0.5).into());
            normals.push((-normal).into());
            uvs.push([phase, 1.0]);

            positions.push((position + normal * ring.width * 0.5).into());
            normals.push(normal.into());
            uvs.push([phase, 0.0]);
        }

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleStrip);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

        mesh
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Cone {
    pub radius: f32,
    pub height: f32,
    pub subdivisions: usize,
}

impl Default for Cone {
    fn default() -> Self {
        Cone {
            radius: 1.0,
            height: 1.0,
            subdivisions: 32,
        }
    }
}

impl From<Cone> for Mesh {
    fn from(cone: Cone) -> Self {
        // code adapted from http://apparat-engine.blogspot.com/2013/04/procedural-meshes-torus.html
        // (source code at https://github.com/SEilers/Apparat)

        let n_vertices = cone.subdivisions + 2;
        let mut positions: Vec<[f32; 3]> = Vec::with_capacity(n_vertices);
        let mut normals: Vec<[f32; 3]> = Vec::with_capacity(n_vertices);
        let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(n_vertices);

        let side_stride = 2.0 * std::f32::consts::PI / cone.subdivisions as f32;

        // Cone tip
        positions.push([0.0, cone.height, 0.0]);
        normals.push(Vec3::Y.into());
        uvs.push([0.0, 1.0]);
        // Bottom center
        positions.push([0.0, 0.0, 0.0]);
        normals.push(Vec3::new(0.0, -1.0, 0.0).into());
        uvs.push([0.0, -1.0]);

        for side in 0..=cone.subdivisions {
            let phi = side_stride * side as f32;
            let x = phi.cos() * cone.radius;
            let y = 0.0;
            let z = phi.sin() * cone.radius;

            let vertex = Vec3::new(x, y, z);
            let tangent = vertex.normalize().cross(Vec3::Y).normalize();
            let edge = (Vec3::Y - vertex).normalize();
            let normal = edge.cross(tangent).normalize();

            positions.push([x, y, z]);
            normals.push(normal.into());
            uvs.push([side as f32 / cone.subdivisions as f32, 0.0]);
        }

        let n_triangles = cone.subdivisions * 2;
        let n_indices = n_triangles * 3;

        let mut indices: Vec<u32> = Vec::with_capacity(n_indices);

        for point in 2..cone.subdivisions + 2 {
            let top = 0;
            let bottom = 1;

            let left = point + 1;
            let right = point;

            indices.push(top as u32);
            indices.push(left as u32);
            indices.push(right as u32);

            indices.push(bottom as u32);
            indices.push(right as u32);
            indices.push(left as u32);
        }

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(Indices::U32(indices)));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh
    }
}
