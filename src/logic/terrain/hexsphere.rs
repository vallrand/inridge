use bevy::prelude::*;
use bevy::math::{Vec3A, Mat3A};
use bevy::render::mesh::{Indices, Mesh, PrimitiveTopology};
use crate::common::noise::perlin::SQRT_3;
use crate::common::geometry::{MeshGeometry, GeometryTopology, subdivide::subdivide, Icosahedron};
use crate::common::geometry::{unwrap_equirectangular, delta_mod};
use crate::common::adjacency::Graph;
use super::grid::MapGridTile;

pub struct HexSphere {
    pub variants: usize,
    pub mesh: MeshGeometry,
    pub graph: Graph,
    pub tiles: Vec<MapGridTile>,
    pub skip_pentagons: bool,
    pub invert: bool,
}

impl From<&HexSphere> for bevy::render::mesh::Mesh {
    fn from(source: &HexSphere) -> Self {
        let dual_vertices = source.mesh.vertices();

        let mut vertices: Vec<[f32; 3]> = Vec::with_capacity(0);
        let mut normals: Vec<[f32; 3]> = Vec::with_capacity(0);
        let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(0);
        let mut colors: Vec<[f32; 4]> = Vec::with_capacity(0);
        let mut indices: Vec<u32> = Vec::with_capacity(0);

        fn calculate_border_uvs(i: usize, neighbors: &[usize], tiles: &Vec<MapGridTile>, colors: &mut Vec<[f32; 4]>, variants: usize){
            const UV_CENTER: (f32, f32) = (0.0, 0.0);
            const UV_LEFT: (f32, f32) = (-1.0,1.0);
            const UV_RIGHT: (f32, f32) = (1.0,1.0);
            const UV_BOTTOM: (f32, f32) = (0.0,-2.0);
            const UV_LEFT_RIGHT: (f32, f32) = (0.0,2.0);
            const UV_LEFT_BOTTOM: (f32, f32) = (-1.0,-1.0);

            colors.push([(tiles[i].variant as f32 + 0.5) / (variants - 1) as f32,0.0,0.0,0.0]);

            let mut border_flag: bool = false;
            for j in 0..neighbors.len() {
                let (prev, next) = (neighbors[j], neighbors[(j+1)%neighbors.len()]);

                let border_uv = match (
                    tiles[i].variant == tiles[prev].variant,
                    tiles[i].variant == tiles[next].variant,
                    border_flag
                ) {
                    (true,true,true) | (true,true,false) => (UV_CENTER.0, UV_CENTER.1, false),
                    (true,false,false) => (UV_LEFT.0, UV_LEFT.1, false),
                    (true,false,true) => {
                        if j == neighbors.len() - 1 {
                            let start_offset = colors.len() - j;
                            if tiles[i].variant == tiles[neighbors[1]].variant {
                                colors[start_offset][2] = UV_BOTTOM.0;
                                colors[start_offset][3] = UV_BOTTOM.1;
                            }else{
                                colors[start_offset][2] = UV_LEFT_BOTTOM.0;
                                colors[start_offset][3] = UV_LEFT_BOTTOM.1;
                            }
                            (UV_BOTTOM.0, UV_BOTTOM.1, true)
                        }else{
                            (UV_RIGHT.0, UV_RIGHT.1, true)
                        }
                    },
                    (false,true,false) => {
                        if j == neighbors.len() - 1 {
                            let end_offset = colors.len() - 1;
                            if tiles[i].variant == tiles[neighbors[j - 1]].variant {
                                colors[end_offset][2] = UV_BOTTOM.0;
                                colors[end_offset][3] = UV_BOTTOM.1;
                            }else{
                                colors[end_offset][2] = UV_LEFT_BOTTOM.0;
                                colors[end_offset][3] = UV_LEFT_BOTTOM.1;
                            }
                            (UV_BOTTOM.0, UV_BOTTOM.1, true)
                        }else{
                            (UV_LEFT.0, UV_LEFT.1, true)
                        }
                    },
                    (false,true,true) => (UV_RIGHT.0, UV_RIGHT.1, false),
                    (false,false,false) => (UV_LEFT.0, UV_LEFT.1, false),
                    (false,false,true) => {
                        if j == neighbors.len() - 1 {
                            (UV_LEFT_RIGHT.0, UV_LEFT_RIGHT.1, true)
                        }else{
                            (UV_RIGHT.0, UV_RIGHT.1, true)
                        }
                    }
                };
                border_flag = border_uv.2;
                colors.push([(tiles[i].variant as f32 + 0.5) / (variants - 1) as f32, 1.0, border_uv.0, border_uv.1]);
            }
        }
        
        for (i, &center) in dual_vertices.iter().enumerate() {
            let neighbors = source.graph.neighbors(i).unwrap_or_default();
            if neighbors.len() != 6 && source.skip_pentagons {
                continue;
            }
            let normal = center.normalize() * if source.invert { -1.0 }else{ 1.0 };
            let vertex_offset = vertices.len() as u32;
            let mut averaged_centroid: Vec3A = Vec3A::ZERO;

            vertices.push(center.to_array());
            normals.push(normal.to_array());
            let uv_anchor = unwrap_equirectangular(&center);
            uvs.push(uv_anchor.clone());

            for j in 0..neighbors.len() {
                let (prev, next) = (neighbors[j], neighbors[(j+1)%neighbors.len()]);
                let (left, right) = (dual_vertices[prev], dual_vertices[next]);
                let average = (center + left + right) / 3.0;
                averaged_centroid += average;
                vertices.push(average.to_array());
                normals.push(normal.to_array());
                let mut uv = unwrap_equirectangular(&average);
                uv[0] = uv_anchor[0] + delta_mod(1.0, uv_anchor[0], uv[0]);
                uv[1] = uv_anchor[1] + delta_mod(1.0, uv_anchor[1], uv[1]);

                uvs.push(uv);
            }
            vertices[vertex_offset as usize] = (averaged_centroid / neighbors.len() as f32).to_array();
            calculate_border_uvs(i, neighbors, &source.tiles, &mut colors, source.variants);
            for j in 0..neighbors.len() as u32 {
                indices.extend_from_slice(&[vertex_offset, vertex_offset + 1 + j, vertex_offset + 1 + (j + 1) % neighbors.len() as u32])
            }
        }
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(Indices::U32(indices)));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        mesh.generate_tangents().unwrap();
        mesh
    }
}

impl HexSphere {
    fn average_hex_transform(index: usize, vertices: &[Vec3A], neighbors: &[usize]) -> Mat4 {
        const LENGTH: usize = 6; const HALF_LENGTH: usize = LENGTH / 2;
        assert_eq!(neighbors.len(), LENGTH);
        let mut corners: [Vec3A; LENGTH] = [Vec3A::ZERO; LENGTH];
        let mut average: Vec3A = Vec3A::ZERO;
        for i in 0..LENGTH {
            corners[i] = (vertices[index] + vertices[neighbors[i]] + vertices[neighbors[(i + 1) % LENGTH]]) / 3.0;
            average += corners[i];
        }
        let mut top_index: usize = 0;
        let mut min_distance: f32 = f32::MAX;
        for i in 0..HALF_LENGTH {
            let centroid = (corners[top_index] + corners[HALF_LENGTH + top_index]) / 2.0;
            let distance_squared = centroid.distance_squared(vertices[index]);
            if min_distance > distance_squared {
                min_distance = distance_squared;
                top_index = i;
            }
        }

        let centroid = (corners[top_index] + corners[HALF_LENGTH + top_index]) / 2.0;
        let index_l = top_index + HALF_LENGTH / 2;
        let index_r = (top_index + HALF_LENGTH + HALF_LENGTH / 2) % LENGTH;

        let middle_left = (corners[index_l] + corners[index_l + 1]) / 2.0;
        let middle_right = (corners[index_r] + corners[(index_r + 1) % LENGTH]) / 2.0;

        let up = corners[top_index] - centroid;
        let right = (middle_left - middle_right) / SQRT_3;
        let normal = vertices[index].normalize() * up.length();

        Mat4::from_cols_array(&[
            right.x, right.y, right.z, 0.0,
            normal.x, normal.y, normal.z, 0.0,
            up.x, up.y, up.z, 0.0,
            centroid.x, centroid.y, centroid.z, 1.0
        ])
    }
    pub fn new(subdivisions: usize, invert: bool) -> Self {
        let mut sphere = subdivide(&Icosahedron{}, subdivisions, crate::common::geometry::subdivide::nlerp);
        sphere *= Mat3A::from_axis_angle(Vec3::X, (1.61803398875 as f32).atan2(1.0));
        if invert { sphere.invert(); }
        let graph = Graph::from(&sphere);
        let tiles: Vec<MapGridTile> = sphere.vertices().iter().enumerate().map(|(index, vertex)|{
            let neighbors = graph.neighbors(index).unwrap();
            if neighbors.len() == 6 {
                let mut transform = Self::average_hex_transform(index, sphere.vertices(), neighbors);
                if invert { transform.y_axis *= -1.0; }
                MapGridTile::from(transform)
            } else {
                let mut normal: Vec3 = vertex.normalize().into();
                if invert { normal *= -1.0; }
                let transform = Transform::from_translation(vertex.clone().into())
                .with_scale(Vec3::splat(Icosahedron::circumscribed_tile_radius(subdivisions) / SQRT_3))
                .looking_to(normal.any_orthonormal_vector(), normal).compute_matrix();
                MapGridTile::from(transform)
            }
        }).collect();
        Self{ mesh: sphere, graph, tiles, variants: 0, skip_pentagons: false, invert }
    }
}