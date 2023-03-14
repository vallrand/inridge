use bevy::prelude::*;
use bevy::math::{Vec3Swizzles};
use bevy::render::mesh::{Indices, Mesh, PrimitiveTopology};

#[derive(Debug, Clone, Copy)]
pub struct QuadSphere {
    pub radius: f32,
    pub resolution: usize,
}

impl Default for QuadSphere {
    fn default() -> Self { Self { radius: 1.0, resolution: 36 } }
}

impl From<QuadSphere> for Mesh {
    fn from(sphere: QuadSphere) -> Self {
        static FACES: [Vec3; 6] = [Vec3::X, Vec3::Y, Vec3::Z, Vec3::NEG_X, Vec3::NEG_Y, Vec3::NEG_Z];
        let resolution: usize = sphere.resolution + 1;
        let total: usize = resolution * resolution * FACES.len();
        let step: f32 = 2.0 / (resolution - 1) as f32;

        let mut vertices: Vec<[f32; 3]> = Vec::with_capacity(total);
        let mut normals: Vec<[f32; 3]> = Vec::with_capacity(total);
        let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(total);
        let mut indices: Vec<u32> = Vec::with_capacity((resolution-1)*(resolution-1)*6*FACES.len());

        for (face, forward) in FACES.iter().enumerate() {
            let up = forward.yzx();
            let right = forward.cross(up);

            for u in 0..resolution {
                for v in 0..resolution {
                    let p = forward.clone()
                    + right * (u as f32 * step - 1.0)
                    + up * (v as f32 * step - 1.0);
                    let p2 = p * p;

                    let unit_vector = p * Vec3::new(
                        (1.0 - 0.5 * (p2.y + p2.z) + p2.y*p2.z / 3.0).sqrt(),
                        (1.0 - 0.5 * (p2.z + p2.x) + p2.z*p2.x / 3.0).sqrt(),
                        (1.0 - 0.5 * (p2.x + p2.y) + p2.x*p2.y / 3.0).sqrt()
                    );
                    normals.push(unit_vector.to_array());
                    vertices.push((unit_vector * sphere.radius).to_array());
                    uvs.push([u as f32 * step, v as f32 * step]);
                }
            }
            let index_offset = face * resolution * resolution;
            for u in 0..resolution-1 {
                for v in 0..resolution-1 {
                    let i: u32 = (v + u * resolution + index_offset) as u32;

                    indices.push(i);
                    indices.push(i + resolution as u32 + 1);
                    indices.push(i + resolution as u32);
                    indices.push(i);
                    indices.push(i + 1);
                    indices.push(i + resolution as u32 + 1);
                }
            }
        }

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(Indices::U32(indices)));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh
    }
}