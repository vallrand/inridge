use bevy::prelude::*;
use crate::common::animation::ease::{Ease, cubic_bezier, cubic_bezier_derivative};
use crate::common::noise::MurMurHash;
use crate::logic::{Agent, MapGrid, FollowingPath, SpatialLookupGrid, BoundingRadius, TargetLock, DegradeImmobilize};

#[derive(serde::Deserialize, Clone, Default, Debug)]
pub enum MovementVariant {
    #[default] None,
    Staggered {
        global_velocity: usize,
        velocity: f32,
        angular_velocity: f32,
        min_distance: f32,
    },
    Float {
        height: f32,
        frequency: f32,
        velocity: f32,
        deviation: f32,
        ease: Ease,
    },
    Probe {
        height: f32,
    },
}

#[derive(Component, Clone, Default)]
pub struct MovementFormation {
    pub time_offset: f32,
    pub prev_direction: Vec3,
    pub next_direction: Vec3,
    pub prev_normal: Vec3,
    pub next_normal: Vec3,
    pub prev_position: Vec3,
    pub next_position: Vec3,
    pub elapsed: f32,
    pub last_reset: bool,
    pub variant: MovementVariant
}
impl MovementFormation {
    pub fn reset(&mut self){
        self.elapsed = 0.0;
        self.prev_normal = self.next_normal;
        self.prev_position = self.next_position;
        self.prev_direction = self.next_direction;
        self.last_reset = true;
    }
    pub fn recalculate_direction(&mut self){
        self.next_direction = self.next_position - self.prev_position;
        let distance = self.next_direction.length();
        self.next_direction = 0.5 * distance * self.next_direction.normalize();
        self.prev_direction = 0.5 * distance * self.prev_direction.normalize();
    }
    pub fn percent(&self) -> f32 {
        match self.variant {
            MovementVariant::Float { ease, .. } => ease.calculate(self.elapsed),
            _ => self.elapsed,
        }
    }
    pub fn sample_position(&self) -> Vec3 {
        match self.variant {
            MovementVariant::Float { .. } | MovementVariant::Probe { .. } => cubic_bezier(
                self.prev_position, 
                self.prev_position + self.prev_direction, 
                self.next_position - self.next_direction,
                self.next_position,
                self.percent()
            ),
            _ => Vec3::lerp(self.prev_position, self.next_position, self.percent()),
        }
    }
    pub fn sample_normal(&self) -> Vec3 {
        Vec3::lerp(self.prev_normal, self.next_normal, self.percent())
    }
    pub fn sample_direction(&self) -> Vec3 {
        match self.variant {
            MovementVariant::Float { .. } | MovementVariant::Probe { .. } => cubic_bezier_derivative(
                self.prev_position, 
                self.prev_position + self.prev_direction, 
                self.next_position - self.next_direction,
                self.next_position,
                self.percent()
            ).normalize(),
            MovementVariant::Staggered { angular_velocity, .. } => Vec3::lerp(
                self.prev_direction, self.next_direction, 
                (self.percent() * angular_velocity).min(1.0)
            ).normalize(),
            _ => self.next_direction
        }
    }
    pub fn sample_rotation(&self) -> Option<Quat> {
        let normal = self.sample_normal();
        let forward = -self.sample_direction().normalize();
        if forward.is_finite() {
            let right = forward.cross(normal).normalize();
            Some(Quat::from_mat3(&Mat3::from_cols(
                forward, right.cross(forward), right
            )).normalize())
        } else {
            None
        }
    }
}

pub fn collision_avoidance(
    lookup: &SpatialLookupGrid<Entity>,
    query_adjacent: &Query<(&Agent, &BoundingRadius, &Transform)>,
    source_entity: Entity,
    parent_transform: &GlobalTransform,
    position: Vec3,
    normal: Vec3,
) -> Vec3 {
    let mut repulse = Vec3::ZERO;
    let Ok((agent, radius, _)) = query_adjacent.get(source_entity) else { return repulse };

    let global_position = parent_transform.transform_point(position);
    for &(entity, center) in lookup.query_around(global_position, 2.0) {
        let Ok((target_agent, target_radius, _)) = query_adjacent.get(entity) else { continue };
        if !target_agent.eq(agent) || entity == source_entity { continue; }
        let difference = global_position - center;
        let distance = difference.length();
        let min_distance = **radius + **target_radius;
        if distance > min_distance { continue; }
        let delta = (min_distance - distance) * (difference / distance);
        repulse += delta;
    }
    repulse = parent_transform.compute_matrix().inverse().transform_vector3(repulse);
    repulse = repulse - normal.dot(repulse) * normal;
    repulse
}

pub fn update_movement_formation(
    time: Res<Time>,
    fixed_time: Res<FixedTime>,
    mut rng: Local<MurMurHash>,
    mut query_unit: Query<(Entity, &Parent, &mut MovementFormation, Option<&FollowingPath>, Option<&TargetLock>, Option<&DegradeImmobilize>)>,
    query_adjacent: Query<(&Agent, &BoundingRadius, &Transform)>,
    query_grid: Query<(&MapGrid, &GlobalTransform)>,
    lookup: Res<SpatialLookupGrid<Entity>>,
){
    let fraction = fixed_time.accumulated().as_secs_f32() / fixed_time.period.as_secs_f32();
    
    for (entity, parent, mut formation, movement, target_lock, immobilize) in query_unit.iter_mut() {
        let Ok((grid, parent_transform)) = query_grid.get(parent.get()) else { continue };
        let decelerate = immobilize.map_or(1.0, |immobilize| 1.0 / (1.0 + **immobilize as f32));
        formation.last_reset = false;

        if formation.is_added() {
            let transform = query_adjacent.get_component::<Transform>(entity).unwrap();
            formation.next_position = transform.translation;
            formation.next_normal = transform.rotation * Vec3::Y;
            formation.next_direction = transform.rotation * Vec3::X;
            formation.reset();
            formation.elapsed = 1.0;
            formation.time_offset = time.elapsed_seconds();
        }

        match formation.variant {
            MovementVariant::None => {
                let Some(movement) = movement else { continue };
                let (prev_index, next_index, percent) = movement.calculate_segment(fraction);

                formation.elapsed = percent;
                formation.prev_position = grid.tiles[prev_index].transform.translation;
                formation.next_position = grid.tiles[next_index].transform.translation;
                formation.prev_normal = grid.tiles[prev_index].normal;
                formation.next_normal = grid.tiles[next_index].normal;
                formation.prev_direction = formation.next_position - formation.prev_position;
                formation.next_direction = formation.prev_direction;
            },
            MovementVariant::Probe { height, .. } => {
                let Some(movement) = movement else { continue };
               
                let (_prev_index, next_index, percent) = movement.calculate_segment(fraction);
                let reset = percent < formation.elapsed;
                formation.elapsed = percent;
                if !reset { continue; }

                formation.reset();
                formation.next_normal = grid.tiles[next_index].normal;
                formation.next_position = grid.tiles[next_index].transform.translation + height * formation.next_normal;
                formation.recalculate_direction();
            },
            MovementVariant::Float { height, deviation, .. } => {
                let Some(movement) = movement else { continue };
                let (prev_index, next_index, percent) = movement.calculate_segment(fraction);
                let reset = percent < formation.elapsed;
                formation.elapsed = percent;

                if !reset { continue; }

                let next_normal = grid.tiles[next_index].normal;
                let prev_position = grid.tiles[prev_index].transform.translation;
                let next_position = grid.tiles[next_index].transform.translation;
                let total_distance = next_position.distance(prev_position);

                formation.reset();
                if formation.prev_position.distance(next_position) < 0.5 * total_distance { continue; }
                formation.next_position = {
                    let mut position = next_position + height * next_normal;

                    rng.next(time.elapsed().as_millis() as u64);
                    let tangent = Vec3::cross(position - formation.prev_position, next_normal).normalize();    
                    position += tangent * (rng.next_f32() * 2.0 - 1.0) * deviation * total_distance;
                    
                    let repulse = collision_avoidance(&lookup, &query_adjacent, entity, &parent_transform, position, next_normal);
                    position += repulse.clamp_length(0.0, total_distance);
                    position
                };
                formation.next_normal = formation.next_position.normalize();
                formation.recalculate_direction();
            },
            MovementVariant::Staggered { velocity, min_distance, .. } => {
                formation.elapsed = (formation.elapsed + decelerate * time.delta_seconds() / velocity).min(1.0);
                if formation.elapsed < 1.0 { continue; }

                if let Some(target) = target_lock {
                    let Ok(target_transform) = query_adjacent.get_component::<Transform>(**target) else { continue };
                    formation.prev_position = formation.next_position;
                    formation.prev_normal = formation.next_normal;
                    formation.elapsed = 0.0;
                    formation.next_position = target_transform.translation;
                    formation.next_normal = target_transform.rotation * Vec3::Y;
                    continue;
                }

                let Some(movement) = movement else { continue };
                let (prev_index, next_index, percent) = movement.calculate_segment(fraction);

                let prev_normal = grid.tiles[prev_index].normal;
                let next_normal = grid.tiles[next_index].normal;
                let prev_position = grid.tiles[prev_index].transform.translation;
                let next_position = grid.tiles[next_index].transform.translation;
        
                let interpolated_position = Vec3::lerp(prev_position, next_position, percent);
                let interpolated_normal = Vec3::lerp(prev_normal, next_normal, percent);
                let distance_squared = formation.next_position.distance_squared(interpolated_position);
                if distance_squared >= min_distance * min_distance {
                    formation.reset();
                    formation.next_position = {
                        let mut position = interpolated_position;
                        rng.next(time.elapsed().as_millis() as u64);
                        let deviation = rng.next_f32() * 2.0 - 1.0;
        
                        let tangent = Vec3::cross(position - formation.prev_position, interpolated_normal).normalize();
        
                        let tile_difference = prev_position.distance(next_position).min(0.5 * min_distance);
        
                        position += tangent * deviation * tile_difference;

                        let repulse = collision_avoidance(&lookup, &query_adjacent, entity, &parent_transform, position, interpolated_normal);
                        position += repulse.clamp_length(0.0, tile_difference);
                        position.normalize()
                    };
                    formation.next_normal = formation.next_position;
                    formation.next_direction = (formation.next_position - formation.prev_position).normalize();
                }
            }
        }
    }
}