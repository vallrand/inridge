use bevy::prelude::*;
use bevy::ecs::query::ReadOnlyWorldQuery;
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use crate::common::spatial::morton::MortonCode;
use crate::common::spatial::aabb::AABB;
use crate::common::geometry::subdivide::PassThroughHasher;

#[derive(Resource, Clone)]
pub struct SpatialLookupGrid<T: Sized> {
    pub cell_size: f32,
    table: HashMap<MortonCode<u64>, usize, BuildHasherDefault<PassThroughHasher>>,
    cells: Vec<Vec<(T, Vec3)>>
}
impl<T: Sized> Default for SpatialLookupGrid<T> { fn default() -> Self { Self::new(1.0) } }
impl<T: Sized> SpatialLookupGrid<T> {
    pub fn new(cell_size: f32) -> Self { Self {
        cell_size,
        table: HashMap::default(),
        cells: Vec::new(),
    } }
    fn cell_position(&self, position: Vec3) -> IVec3 { (position / self.cell_size).as_ivec3() }
    fn index(&self, cell: IVec3) -> MortonCode<u64> {
        MortonCode::<u64>::from(UVec3::new(
            if cell.x < 0 { (-cell.x as u32) << 1 + 1 }else{ (cell.x as u32) << 1 },
            if cell.y < 0 { (-cell.y as u32) << 1 + 1 }else{ (cell.y as u32) << 1 },
            if cell.z < 0 { (-cell.z as u32) << 1 + 1 }else{ (cell.z as u32) << 1 },
        ))
    }
    pub fn refresh(&mut self){
        for list in self.cells.iter_mut() {
            list.clear();
        }
    }
    pub fn insert(&mut self, position: Vec3, key: T){
        let code = self.index(self.cell_position(position));
        let index = self.table.entry(code).or_insert_with(||{
            self.cells.push(Vec::new());
            self.cells.len() - 1
        });
        self.cells[*index].push((key, position));
    }
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = &'a (T, Vec3)> {
        self.cells.iter().flat_map(|list|list.iter())
    }
    pub fn query_around<'a>(&'a self, position: Vec3, radius: f32) -> impl Iterator<Item = &'a (T, Vec3)> {
        let aabb = AABB::from(position) + radius;
        let radius_squared = radius * radius;
        self.query_aabb(aabb).filter(move |entry|{
            position.distance_squared(entry.1) <= radius_squared
        })
    }
    pub fn query_aabb<'a>(&'a self, aabb: AABB) -> impl Iterator<Item = &'a (T, Vec3)> {
        let min = self.cell_position(aabb.min.into());
        let max = self.cell_position(aabb.max.into());
        RangeAABB { min, max, i: min }
        .flat_map(move |i|self.table.get(&self.index(i)).map(
            |index|self.cells[*index].as_slice()
        )).flat_map(|list|list.iter())
    }
}

pub struct RangeAABB {
    min: IVec3,
    max: IVec3,
    i: IVec3,
}
impl Iterator for RangeAABB {
    type Item = IVec3;
    fn next(&mut self) -> Option<Self::Item> {
        if self.i.x > self.max.x {
            self.i.x = self.min.x;
            self.i.y += 1;
        }
        if self.i.y > self.max.y {
            self.i.y = self.min.y;
            self.i.z += 1;
        }
        if self.i.z > self.max.z {
            return None
        }
        let value = self.i;
        self.i.x += 1;
        Some(value)
    }
}

pub fn update_spatial_lookup_grid<T: ReadOnlyWorldQuery>(
    mut aabb: Local<AABB>,
    mut lookup: ResMut<SpatialLookupGrid<Entity>>,
    query: Query<(Entity, &GlobalTransform), T>
){
    const PARTITIONS: usize = 8;
    let size = aabb.size();
    let largest = size.x.max(size.y).max(size.z).max(PARTITIONS as f32);
    lookup.cell_size = largest / PARTITIONS as f32;

    lookup.refresh();
    for (entity, transform) in query.iter() {
        let position = transform.translation();
        *aabb += position;
        lookup.insert(position, entity);
    }
}