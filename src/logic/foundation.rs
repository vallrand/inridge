use bevy::prelude::*;
use std::ops::AddAssign;
use crate::common::animation::ease::lerp;

#[derive(Component, serde::Deserialize, Clone, Default, Debug)]
pub struct Integrity {
    pub max: i32,
    pub rate: i32,
    #[serde(default, skip)] pub prev_max: i32,
    #[serde(default, skip)] pub absorbed: i32,
    #[serde(default, skip)] pub restored: i32,
    #[serde(default, skip)] pub prev_restored: i32,
}

impl AddAssign<Integrity> for Integrity {
    fn add_assign(&mut self, rhs: Integrity) {
        self.prev_max = self.max;
        self.max = rhs.max;

        self.restored = if self.rate == 0 { 0 } else {
            (self.restored / self.rate) * rhs.rate + self.restored % self.rate
        };
        self.prev_restored = if self.rate == 0 { 0 } else {
            (self.prev_restored / self.rate) * rhs.rate + self.prev_restored % self.rate
        };
        self.rate = rhs.rate;
    }
}

impl Integrity {
    pub fn apply_damage(&mut self, damage: i32){
        self.absorbed = self.absorbed.checked_add(damage).unwrap_or(i32::MAX);
    }
    pub fn get_restored(&self, fraction: f32) -> f32 {
        if self.rate == 0 { 0.0 } else {
            lerp(self.prev_restored as f32, self.restored as f32, fraction) / self.rate as f32
        }
    }
    pub fn get_capacity(&self, construction: f32) -> f32 {
        lerp(self.prev_max as f32, self.max as f32, construction)
    }
    pub fn calculate_damage(&self, fraction: f32, construction: f32) -> f32 {
        let restored = self.get_restored(fraction);
        let damaged = self.absorbed as f32 - restored;
        let max = self.get_capacity(construction);

        if max == 0.0 { 0.0 }else{ damaged / max }
    }
    pub fn calculate(&self, fraction: f32, construction: f32) -> f32 {
        let restored = self.get_restored(fraction);
        let damaged = self.absorbed as f32 - restored;
        let max = self.get_capacity(construction);

        (max - damaged) / self.max as f32
    }
    pub fn tier(&self) -> i32 { self.max / 10 }
}

#[derive(Component, serde::Deserialize, Clone, Default, Debug)]
pub struct UnderConstruction {
    pub required: i32,
    #[serde(default, skip)] pub matter_consumed: i32,
    #[serde(default, skip)] prev_matter_consumed: Option<i32>,
}
impl UnderConstruction {
    pub fn active(&self) -> bool { self.prev_matter_consumed.map_or(true,
        |prev_matter_consumed|self.matter_consumed > prev_matter_consumed)
    }
    pub fn calculate(&self, fraction: f32) -> f32 {
        lerp(
            self.prev_matter_consumed.unwrap_or_default() as f32, self.matter_consumed as f32, fraction
        ) / self.required as f32
    }
    pub fn tier(&self) -> i32 { 1 }
}

#[derive(Component, Clone, Default, Debug)]
pub struct Suspended;

use crate::logic::{MapGrid, GridTileIndex, ConstructionEvent, CombatEvent, GroupLink};
pub fn construction_phase(
    mut events: EventWriter<ConstructionEvent>,
    mut commands: Commands,
    mut query_grid: Query<&mut MapGrid>,
    mut query_unit: Query<(Entity, &Parent, &GridTileIndex, &mut UnderConstruction)>
){
    for (entity, parent, tile_index, mut construction) in query_unit.iter_mut() {
        construction.prev_matter_consumed = Some(construction.matter_consumed);

        if construction.matter_consumed < construction.required { continue; }
        let Ok(mut grid) = query_grid.get_mut(parent.get()) else { continue };
        commands.entity(entity).remove::<UnderConstruction>();
        let added = grid.tiles[**tile_index].flags & MapGrid::OWNERSHIP != 0;
        grid.tiles[**tile_index].flags |= MapGrid::OWNERSHIP | MapGrid::BLOCKER;
        events.send(ConstructionEvent::Assemble { entity, parent: parent.get(), index: **tile_index, extend: !added });
    }
}

pub fn destruction_phase(
    mut events: EventWriter<ConstructionEvent>,
    mut events_combat: EventWriter<CombatEvent>,
    mut commands: Commands,
    mut query_grid: Query<&mut MapGrid>,
    mut query_unit: Query<(Entity, &Parent, &GridTileIndex, Option<&GroupLink>, &mut Integrity, Option<&UnderConstruction>)>,
){
    for (entity, parent, tile_index, group, mut integrity, construction) in query_unit.iter_mut() {
        integrity.prev_restored = integrity.restored;
        let construction_percent = construction.map_or(1.0,|construction|construction.calculate(1.0));
        let fraction = integrity.calculate(0.0, construction_percent);
        if fraction < 0.0 {
            let Ok(mut grid) = query_grid.get_mut(parent.get()) else { continue };
            if group.is_some() {
                grid.tiles[**tile_index].clear();
                events.send(ConstructionEvent::Dismantle { entity, parent: parent.get(), index: **tile_index });
            }
            events_combat.send(CombatEvent::Destruct(entity));

            commands.entity(entity).remove::<(GroupLink, Integrity, UnderConstruction)>();
        }
    }
}

pub fn reconstruction_phase(
    mut query_unit: Query<&mut Integrity, (With<GroupLink>, Without<UnderConstruction>, Without<Suspended>)>
){
    for mut integrity in query_unit.iter_mut() {
        if integrity.rate > 0 && integrity.restored / integrity.rate < integrity.absorbed {
            integrity.restored += 1;
        }
    }
}