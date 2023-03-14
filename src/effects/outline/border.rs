use bevy::prelude::*;
use std::collections::BTreeSet;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::render::mesh::{Indices, Mesh};
use crate::logic::MapGrid;

#[derive(Clone, Default, Debug)]
pub struct BorderOutline {
    pub width: f32,
    pub alignment: f32,
    pub flip: bool,
    pub closed: bool,
    pub v_range: Vec2,
    pub positions: Vec<Vec<Vec3>>,
    pub normals: Vec<Vec<Vec3>>,
}
impl BorderOutline {
    pub fn from_directions(grid: &MapGrid, path: &[usize]) -> Self {
        let mut positions: Vec<Vec<Vec3>> = Vec::with_capacity(path.len() - 1);
        let mut normals: Vec<Vec<Vec3>> = Vec::with_capacity(path.len() - 1);
        for pair in path.windows(2) {
            let prev_position = grid.tiles[pair[0]].transform.translation;
            let next_position = grid.tiles[pair[1]].transform.translation;

            positions.push(vec![
                (prev_position + (prev_position - next_position) / 2.0).normalize(),
                ((prev_position + next_position) / 2.0).normalize()
            ]);
            normals.push(vec![grid.tiles[pair[0]].normal; 2]);
        }
        Self { positions, normals, closed: false, v_range: Vec2::Y, ..Default::default() }
    }
    pub fn from_path(grid: &MapGrid, path: &[usize]) -> Self {
        let mut positions: Vec<Vec3> = Vec::with_capacity(path.len());
        let mut normals: Vec<Vec3> = Vec::with_capacity(path.len());
        for &index in path.iter() {
            positions.push(grid.tiles[index].transform.translation);
            normals.push(grid.tiles[index].normal);
        }
        Self { positions: vec![positions], normals: vec![normals], closed: false, v_range: Vec2::Y, ..Default::default() }
    }
    pub fn from_single(grid: &MapGrid, index: usize) -> Self {
        let neighbors = grid.graph.neighbors(index).unwrap();

        let mut positions: Vec<Vec3> = Vec::with_capacity(neighbors.len());
        let normals: Vec<Vec3> = vec![grid.tiles[index].normal; neighbors.len()];

        for i in 0..neighbors.len() {
            let (prev, next) = (neighbors[i], neighbors[(i+1)%neighbors.len()]);
            positions.push((
                grid.tiles[index].transform.translation +
                grid.tiles[prev].transform.translation +
                grid.tiles[next].transform.translation
            ).normalize());
        }
        Self { positions: vec![positions], normals: vec![normals], closed: true, v_range: Vec2::Y, ..Default::default() }
    }
    pub fn from_group(grid: &MapGrid, indices: &[usize], expand: bool) -> Self {
        let mut visited = bevy::utils::HashSet::<usize>::new();
        fn edge_key(index_left: usize, index_right: usize, index_max: usize) -> usize {
            if index_left < index_right { index_left * index_max + index_right }
            else { index_right * index_max + index_left }
        }
        let mut positions: Vec<Vec<Vec3>> = Vec::new();
        let mut normals: Vec<Vec<Vec3>> = Vec::new();
        let index_set = if expand { None }else{ Some(BTreeSet::from_iter(indices.iter().cloned())) };

        for &tile_index in indices.iter() {
            let group = grid.visited.get(&tile_index).unwrap();
            let mut neighbors = grid.graph.neighbors(tile_index).unwrap();
            for i in 0..neighbors.len() {
                let key = edge_key(tile_index, neighbors[i], grid.tiles.len());
                if visited.contains(&key) { continue; }

                let edge = match &index_set {
                    Some(index_set) => !index_set.contains(&neighbors[i]),
                    None => grid.visited.get(&neighbors[i]).map(|i|i!=group).unwrap_or(true)
                };
                if !edge { continue; }
                let mut contour: Vec<Vec3> = Vec::new();

                let mut pivot = tile_index;
                let mut j = i;
                loop {
                    if match &index_set {
                        Some(index_set) => !index_set.contains(&neighbors[j]),
                        None => grid.visited.get(&neighbors[j]).map(|i|i!=group).unwrap_or(true)
                    } {
                        visited.insert(edge_key(pivot, neighbors[j], grid.tiles.len()));
                        let prev_index = neighbors[j];
                        j = (j+1)%neighbors.len();

                        contour.push((
                            grid.tiles[pivot].transform.translation +
                            grid.tiles[prev_index].transform.translation +
                            grid.tiles[neighbors[j]].transform.translation
                        ).normalize());
                    } else {
                        let prev_pivot = pivot;
                        let prev_index = neighbors[(j + neighbors.len() - 1) % neighbors.len()];
                        pivot = neighbors[j];
                        neighbors = grid.graph.neighbors(pivot).unwrap();
                        j = neighbors.iter().position(|other|*other==prev_pivot).unwrap();
                        j = (j + 1) % neighbors.len();
                        assert_eq!(prev_index, neighbors[j]);
                    }
                    if pivot == tile_index && j == i { break; }
                }

                normals.push(contour.iter().map(|n|n.normalize()).collect::<Vec<Vec3>>());
                positions.push(contour);
            }
        }
        Self { positions, normals, closed: true, v_range: Vec2::Y, ..Default::default() }
    }
    pub fn with_offset(mut self, offset: f32) -> Self {
        for i in 0..self.positions.len() {
            for j in 0..self.positions[i].len() {
                self.positions[i][j] += self.normals[i][j] * offset;
            }
        }
        self
    }
    pub fn with_uv(mut self, v0: f32, v1: f32) -> Self {
        self.v_range = Vec2::new(v0, v1);
        self
    }
    pub fn with_stroke(mut self, width: f32, alignment: f32, flip: bool) -> Self {
        self.width = width;
        self.alignment = alignment;
        self.flip = flip;
        self
    }
}
impl From<BorderOutline> for Mesh {
    fn from(source: BorderOutline) -> Self {
        let mut positions: Vec<[f32; 3]> = Vec::with_capacity(0);
        let mut normals: Vec<[f32; 3]> = Vec::with_capacity(0);
        let mut tangents: Vec<[f32; 4]> = Vec::with_capacity(0);
        let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(0);
        let mut indices: Vec<u32> = Vec::with_capacity(0);

        let mut index_offset: u32 = 0;
        for contour in 0..source.positions.len() {
            let source_positions = &source.positions[contour];
            let source_normals = &source.normals[contour];

            let last_index: usize = source_positions.len() - 1;
            for i in 0..=last_index {
                let prev = if i > 0 { i - 1 }else if source.closed { last_index }else{ 0 };
                let next = if i < last_index { i + 1 }else if source.closed { 0 }else { last_index };
                let binormal = (source_positions[next] - source_positions[prev]).normalize_or_zero();
                let (normal, tangent) = if source.flip {
                    (source_normals[i].cross(binormal), source_normals[i])
                } else {
                    (source_normals[i], source_normals[i].cross(binormal))
                };

                positions.push((source_positions[i] - tangent * source.width * source.alignment).to_array());
                positions.push((source_positions[i] + tangent * source.width * (1.0 - source.alignment)).to_array());
                normals.extend_from_slice(&[normal.to_array(); 2]);
                tangents.extend_from_slice(&[tangent.extend(0.0).to_array(); 2]);
                
                uvs.extend_from_slice(&[[i as f32, source.v_range.x], [i as f32, source.v_range.y]]);
            }
            if source.closed {
                positions.extend_from_within(index_offset as usize..index_offset as usize+2);
                normals.extend_from_within(index_offset as usize..index_offset as usize+2);
                tangents.extend_from_within(index_offset as usize..index_offset as usize+2);
                uvs.extend_from_slice(&[[(last_index + 1) as f32, source.v_range.x], [(last_index + 1) as f32, source.v_range.y]]);
            }
            for i in 0..last_index as u32 + source.closed as u32 {
                indices.extend_from_slice(&[
                    index_offset + i * 2 + 0, index_offset + i * 2 + 2, index_offset + i * 2 + 1,
                    index_offset + i * 2 + 1, index_offset + i * 2 + 2, index_offset + i * 2 + 3,
                ]);
            }
            index_offset += last_index as u32 * 2 + 2 + 2 * source.closed as u32;
        }

        let mut mesh: Mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(Indices::U32(indices)));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_TANGENT, tangents);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh
    }
}