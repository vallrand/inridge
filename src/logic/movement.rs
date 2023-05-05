use bevy::prelude::*;
use super::{MapGrid, GridTileIndex, GroupLink, ConstructionEvent, Integrity, DegradeImmobilize, MilitarySupply};
use crate::effects::animation::MovementFormation;

#[derive(Component, serde::Deserialize, Deref, DerefMut, Clone, Default, Debug)]
pub struct BoundingRadius(f32);

#[derive(Component, serde::Deserialize, Deref, DerefMut, Clone, Copy, Default, Debug)]
pub struct Velocity(pub i32);

#[derive(Component, Clone, Default)]
pub struct LandingProbe;

#[derive(Component, Clone, Default)]
pub struct FollowingPath {
    pub path: Vec<usize>,
    pub elapsed: f32,
    pub prev_elapsed: f32,
    pub last_step: u32,
}
impl From<Vec<usize>> for FollowingPath {
    fn from(path: Vec<usize>) -> Self { Self { path, ..Default::default() } }
}
impl FollowingPath {
    pub fn update(&mut self, velocity: f32){
        self.prev_elapsed = self.elapsed;
        self.last_step += 1;
        if velocity > 0.0 { self.elapsed += 1.0 / velocity; }
    }
    pub fn calculate_segment(&self, fraction: f32) -> (usize, usize, f32) {
        let segments = self.path.len() - 1;
        let elapsed = self.prev_elapsed + fraction * (self.elapsed - self.prev_elapsed);
        if elapsed <= 0.0 {
            (self.path[0], self.path[segments.min(1)], 0.0)
        }else if elapsed >= segments as f32 {
            (self.path[segments.max(1) - 1], self.path[segments], 1.0)
        } else {
            let prev_index = elapsed as usize;
            let next_index = (prev_index + 1).min(segments);
            (self.path[prev_index], self.path[next_index], elapsed.fract())
        }
    }
    pub fn stepped_over(&self) -> bool { self.next() > self.prev() }
    pub fn just_started(&self) -> bool { self.elapsed == 0.0 && self.prev_elapsed == 0.0 }
    pub fn has_ended(&self) -> bool { self.next_index().is_none() }
    pub fn next_index(&self) -> Option<&usize> { self.path.get(self.next() + 1) }
    pub fn next(&self) -> usize { (self.elapsed.max(0.0) as usize).min(self.path.len() - 1) }
    pub fn prev(&self) -> usize { (self.prev_elapsed.max(0.0) as usize).min(self.path.len() - 1) }
}

pub fn execute_movement_directives(
    mut commands: Commands,
    mut query_unit: Query<(Entity, &mut FollowingPath, &mut GridTileIndex, &Velocity, Option<&DegradeImmobilize>), With<MovementFormation>>,
){
    for (
        entity, mut movement, mut tile_index, velocity, immobilize
    ) in query_unit.iter_mut() {
        if movement.stepped_over() || movement.just_started() {
            movement.last_step = 0;
            if let Some(&next_index) = movement.next_index() {
                tile_index.0 = next_index;
            } else {
                commands.entity(entity).remove::<FollowingPath>();
            }
        }
        let mut velocity = **velocity;
        if let Some(immobilize) = immobilize {
            velocity *= 1 + immobilize.0;
        }
        movement.update(velocity as f32);
    }
}

use crate::logic::{GlobalEconomy, PriorityOrder};

pub fn execute_probe_landing(
    mut commands: Commands,
    mut global: ResMut<GlobalEconomy>,
    mut events: EventWriter<ConstructionEvent>,
    mut query_unit: Query<(Entity, &Parent, &GridTileIndex, &mut Integrity), (Without<FollowingPath>, With<LandingProbe>)>,
    mut query_grid: Query<&mut MapGrid>,
){
    for (entity, parent, tile_index, mut integrity) in query_unit.iter_mut() {
        let Ok(mut grid) = query_grid.get_mut(parent.get()) else { continue };
        commands.entity(entity).remove::<LandingProbe>();
        if grid.tiles[tile_index.0].is_empty() {
            grid.tiles[tile_index.0].set_entity(entity);

            commands.entity(entity)
                .insert(PriorityOrder(global.next_priority()))
                .insert(GroupLink::default());

            events.send(ConstructionEvent::Assemble { entity, parent: parent.get(), index: **tile_index, extend: false });
        } else {
            integrity.apply_damage(i32::MAX);
        }
    }
}

pub fn execute_structure_relocation(
    mut events: EventWriter<ConstructionEvent>,
    mut commands: Commands,
    mut query_grid: Query<&mut MapGrid>,
    mut query_unit: ParamSet<(
        Query<(Entity, &Parent, &mut FollowingPath, &mut GridTileIndex, &Velocity), (Without<MovementFormation>, With<Integrity>)>,
        Query<(Entity, &Parent, &FollowingPath), (Without<Integrity>, Without<MovementFormation>)>,
    )>,
){
    for (entity, parent, movement) in query_unit.p1().iter() {
        commands.entity(entity).remove::<FollowingPath>();
        let Ok(mut grid) = query_grid.get_mut(parent.get()) else { continue };
        let step = movement.prev();

        grid.tiles[movement.path[step]].clear();
        if let Some(&next_index) = movement.path.get(step + 1) {
            grid.tiles[next_index].clear();
        }
    }
    for (entity, parent, mut movement, mut tile_index, velocity) in query_unit.p0().iter_mut() {
        let Ok(mut grid) = query_grid.get_mut(parent.get()) else { continue };
        if movement.just_started() {
            commands.entity(entity).remove::<GroupLink>().remove::<MilitarySupply>();
            events.send(ConstructionEvent::Dismantle { entity, parent: parent.get(), index: **tile_index });
        }

        if movement.stepped_over() || movement.just_started() {
            movement.last_step = 0;
            let step = movement.next();
            if step > 0 { grid.tiles[movement.path[step - 1]].clear(); }

            let prev_index = movement.path[step];
            if let Some(&next_index) = movement.path.get(step + 1)
            .filter(|index|grid.tiles[**index].is_empty()) {
                tile_index.0 = next_index;

                grid.tiles[prev_index].reference = None;
                grid.tiles[prev_index].flags = MapGrid::BLOCKER;
                grid.tiles[next_index].flags = MapGrid::BLOCKER;
                grid.tiles[next_index].reference = Some(entity);
            } else {
                grid.tiles[**tile_index].flags = MapGrid::BLOCKER | MapGrid::OWNERSHIP;
                grid.tiles[**tile_index].reference = Some(entity);

                commands.entity(entity).remove::<FollowingPath>()
                    .insert(GroupLink::default());

                events.send(ConstructionEvent::Assemble { entity, parent: parent.get(), index: **tile_index, extend: false });
            }
        }
        movement.update(**velocity as f32);
    }
}