use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use std::f32::consts::TAU;

pub struct Cylinder {
    pub radius: f32,
    pub height: f32,
    pub resolution: u32,
    pub segments: u32,
    pub cap_upper: bool,
    pub cap_lower: bool,
}

impl Default for Cylinder {
    fn default() -> Self { Self {
        radius: 0.5,
        height: 1.0,
        resolution: 16,
        segments: 1,
        cap_upper: true,
        cap_lower: true,
    } }
}

impl From<Cylinder> for Mesh {
    fn from(shape: Cylinder) -> Self {
        let num_caps = shape.cap_upper as u32 + shape.cap_lower as u32;
        let num_rings = shape.segments + 1;
        let num_vertices = shape.resolution * num_caps + num_rings * (shape.resolution + 1);
        let num_faces = shape.resolution * (num_rings - 2);
        let num_indices = (2 * num_faces + num_caps * (shape.resolution - 1) * 2) * 3;

        let mut positions = Vec::with_capacity(num_vertices as usize);
        let mut normals = Vec::with_capacity(num_vertices as usize);
        let mut uvs = Vec::with_capacity(num_vertices as usize);
        let mut indices = Vec::with_capacity(num_indices as usize);

        let step_theta = TAU / shape.resolution as f32;
        let step_y = shape.height / shape.segments as f32;
        for ring in 0..num_rings {
            let y = -shape.height / 2.0 + ring as f32 * step_y;

            for segment in 0..=shape.resolution {
                let theta = segment as f32 * step_theta;
                let (sin, cos) = theta.sin_cos();

                positions.push([shape.radius * cos, y, shape.radius * sin]);
                normals.push([cos, 0., sin]);
                uvs.push([
                    segment as f32 / shape.resolution as f32,
                    ring as f32 / shape.segments as f32,
                ]);
            }
        }

        for i in 0..shape.segments {
            let ring = i * (shape.resolution + 1);
            let next_ring = (i + 1) * (shape.resolution + 1);
            for j in 0..shape.resolution {
                indices.extend_from_slice(&[
                    ring + j, next_ring + j, ring + j + 1,
                    next_ring + j, next_ring + j + 1, ring + j + 1,
                ]);
            }
        }

        let mut build_cap = |top: bool| {
            let offset = positions.len() as u32;
            let (y, normal_y, winding) = if top {
                (shape.height / 2., 1., (1, 0))
            } else {
                (shape.height / -2., -1., (0, 1))
            };

            for i in 0..shape.resolution {
                let theta = i as f32 * step_theta;
                let (sin, cos) = theta.sin_cos();

                positions.push([cos * shape.radius, y, sin * shape.radius]);
                normals.push([0.0, normal_y, 0.0]);
                uvs.push([0.5 * (cos + 1.0), 1.0 - 0.5 * (sin + 1.0)]);
            }

            for i in 1..(shape.resolution - 1) {
                indices.extend_from_slice(&[
                    offset,
                    offset + i + winding.0,
                    offset + i + winding.1,
                ]);
            }
        };

        if shape.cap_upper { build_cap(true); }
        if shape.cap_lower { build_cap(false); }

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(Indices::U32(indices)));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh
    }
}
