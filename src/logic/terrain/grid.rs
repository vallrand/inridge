use bevy::prelude::*;
use bevy::utils::HashMap;
use std::collections::VecDeque;
use crate::common::adjacency::Graph;

#[derive(Component, Deref, DerefMut, Clone, Copy, PartialEq, Eq)]
pub struct GridTileIndex(pub usize);

#[derive(Clone, Default, Debug)]
pub struct MapGridTile {
    pub transform: Transform,
    pub normal: Vec3,

    pub variant: usize,
    pub flags: u8,
    pub reference: Option<Entity>
}

impl From<Mat4> for MapGridTile {
    fn from(matrix: Mat4) -> Self { Self {
        normal: matrix.transform_vector3(Vec3::Y).normalize(),
        transform:  Transform::from_matrix(matrix),
        ..Default::default()
    } }
}

impl MapGridTile {
    pub fn is_empty(&self) -> bool { self.flags == 0 && self.reference.is_none() }
    pub fn set_entity(&mut self, entity: Entity){
        self.flags = MapGrid::BLOCKER | MapGrid::OWNERSHIP;
        self.reference = Some(entity);
    }
    pub fn clear(&mut self){
        self.flags = 0;
        self.reference = None;
    }
}

#[derive(Component, Clone, Default)]
pub struct MapGrid {
    pub tiles: Vec<MapGridTile>,
    pub graph: Graph,
    pub visited: HashMap<usize, usize>,
}

impl MapGrid {
    pub const BLOCKER: u8 = 0b0001;
    pub const OWNERSHIP: u8 = 0b0010;
    pub fn find_closest(&self, mut start_index: usize, target_position: Vec3) -> usize {
        loop {
            let mut min_index = start_index;
            let mut min_distance_squared = self.tiles[start_index].transform.translation.distance_squared(target_position);
            let Some(neighbors) = self.graph.neighbors(start_index) else { break start_index };
            for &neighbor in neighbors {
                let distance_squared = self.tiles[neighbor].transform.translation.distance_squared(target_position);
                if distance_squared < min_distance_squared {
                    min_distance_squared = distance_squared;
                    min_index = neighbor;
                }
            }
            start_index = if min_index != start_index { min_index }else{ break start_index };
        }
    }
    pub fn iter_adjacent_groups<'a>(&'a self, index: usize) -> impl Iterator<Item = &'a usize> {
        self.graph.neighbors(index).unwrap_or_default().iter()
        .filter(|&i|self.tiles[*i].flags & MapGrid::OWNERSHIP != 0)
        .filter_map(|i|self.visited.get(i))
    }
    pub fn fill_group<T>(&mut self, index: usize, group_index: usize, mut filter: impl FnMut(&MapGridTile) -> Option<T>) -> Option<Vec<T>> {
        if self.tiles[index].flags & MapGrid::OWNERSHIP == 0 { return None; }
        let mut out: Vec<T> = vec![filter(&self.tiles[index])?];
        let mut stack: VecDeque<usize> = VecDeque::new();
        self.visited.insert(index, group_index);
        stack.push_back(index);
        loop {
            let Some(index) = stack.pop_front() else { break Some(out) };
            let Some(neighbors) = self.graph.neighbors(index) else { continue };
            for &neighbor in neighbors.iter() {
                if self.visited.contains_key(&neighbor) { continue; }
                let Some(value) = filter(&self.tiles[neighbor]) else { continue };
                out.push(value);
                self.visited.insert(neighbor, group_index);
                if self.tiles[neighbor].flags & MapGrid::OWNERSHIP != 0 {
                    stack.push_back(neighbor);
                }
            }
        }
    }
}