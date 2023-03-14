use std::time::Duration;
use bevy::prelude::*;
use bevy::utils::FloatOrd;
use bevy::ecs::query::ReadOnlyWorldQuery;
use crate::extensions::CommandsExtension;
use crate::interaction::ActionSelector;
use crate::logic::{CombatEvent, SpatialLookupGrid, UpgradeVariant, DegradeImmobilize};

#[derive(serde::Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum UnitDirective {
    Relocate,
    OpenGate,
}

impl From<UnitDirective> for ActionSelector {
    fn from(value: UnitDirective) -> Self { match value {
        UnitDirective::Relocate { .. } => ActionSelector::FollowPath(None),
        UnitDirective::OpenGate => ActionSelector::Target(None),
    } }
}
#[derive(Component, Clone, Default)]
pub struct MilitarySupply {
    pub snapshot: bool,
    pub range: i32,
    pub amplitude: i32,
    pub frequency: i32,
}
impl MilitarySupply {
    pub fn range_multipler(&self) -> f32 { 1.0 + self.range as f32 / 3.0 }
    pub fn rate_multiplier(&self) -> f32 { 1.0 / (1.0 + self.frequency as f32) }
}

#[derive(Component, serde::Deserialize, Clone, Debug)]
pub enum MilitaryBinding {
    Trajectory {
        key: String,
        axis: Option<Mat3>,
        angular_limit: f32,
        vertical_limit: f32,
        radius: (f32, f32),
        cooldown: f32,
        damage: i32,
        #[serde(default, skip)] cooldown_timer: Timer,
        #[serde(default, skip)] orientation: Quat,
    },
    Connection {
        radius: (f32, f32),
        limit: i32,
        damage: i32,
        rate: f32,
        degrade: Option<UpgradeVariant>,
        #[serde(default, skip)] released: i32,
    },
    Area {
        radius: (f32, f32),
        degrade: Option<UpgradeVariant>,
    },
    Impact {
        radius: (f32, f32),
        area: f32,
        damage: i32,
    },
}
impl MilitaryBinding {
    pub fn radius(&self) -> f32 { match self {
        MilitaryBinding::Trajectory { radius, .. } |
        MilitaryBinding::Connection { radius, .. } |
        MilitaryBinding::Area { radius, .. } |
        MilitaryBinding::Impact { radius, .. } => radius.1,
    } }
    pub fn is_close_range(&self) -> bool { match self {
        MilitaryBinding::Impact { .. } => true,
        _ => false
    } }
}

#[derive(Component, Deref, DerefMut, Clone)]
pub struct TargetLock(Entity);

#[derive(Component, Deref, DerefMut, Clone)]
pub struct SourceLink(Entity);

#[derive(serde::Deserialize, Component, Clone)]
pub struct TrajectoryEffect {
    pub linked: bool,
    pub intro: Timer,
    pub outro: Timer,
}

#[derive(serde::Deserialize, Component, Clone, Debug)]
pub enum ImpactEffect {
    Single {
        interval: Timer,
        damage: i32,
    },
    Area {
        interval: Timer,
        damage: i32,
        radius: f32,
    }
}

use crate::logic::{Agent, Suspended, UnderConstruction, MatterBinding, Integrity, FollowingPath};
use crate::logic::{UpgradeAmplitude, UpgradeFrequency, UpgradeRange};

pub fn apply_combat_damage(
    time: Res<Time>,
    lookup: Res<SpatialLookupGrid<Entity>>,
    mut commands: Commands,
    mut events: EventWriter<CombatEvent>,
    mut query_unit: Query<&mut Integrity>,
    mut query_source: Query<&mut MilitaryBinding>,
    query_supply: Query<&MilitarySupply>,
    mut query: ParamSet<(
        Query<(Entity, Option<&SourceLink>, &mut TrajectoryEffect), Without<ImpactEffect>>,
        Query<(Entity, &TargetLock, Option<&SourceLink>, &mut ImpactEffect, Option<&mut TrajectoryEffect>)>,
    )>,
    query_target: Query<(&Agent, &GlobalTransform)>
){
    for (entity, source, mut trajectory) in query.p0().iter_mut() {
        trajectory.outro.tick(time.delta());
        if !trajectory.outro.finished() { continue; }

        if let Some(MilitaryBinding::Connection { released, .. }) = source.and_then(|entity|
            query_source.get_mut(**entity).ok()
        ).as_deref_mut() {
            *released -= 1;
        }

        commands.entity(entity).despawn_recursive();
    }
    for (entity, target, source, mut impact, mut trajectory) in query.p1().iter_mut() {
        let relevant = match (source, trajectory.as_ref()) {
            (Some(entity), Some(trajectory)) => !trajectory.linked || query_supply.contains(**entity),
            _ => true
        };
        if relevant {
            if let Some(trajectory) = trajectory.as_deref_mut() {
                trajectory.intro.tick(time.delta());
                if !trajectory.intro.finished() { continue; }
            }
            match impact.as_mut() {
                ImpactEffect::Single { interval, damage } => {
                    interval.tick(time.delta());
                    if !interval.just_finished() { continue; }
                    if let Ok(mut integrity) = query_unit.get_mut(**target) {
                        integrity.apply_damage(*damage);
                        if interval.mode() == TimerMode::Once {
                            events.send(CombatEvent::ProjectileHit(entity, **target));
                        }
                        if interval.mode() == TimerMode::Repeating { continue; }
                    }
                },
                ImpactEffect::Area { interval, damage, radius } => {
                    interval.tick(time.delta());
                    if !interval.just_finished() { continue; }

                    if let Some(mut integrity) = source.and_then(|source|query_unit.get_mut(source.0).ok()) {
                        integrity.apply_damage(i32::MAX);
                    }

                    let Ok((agent, transform)) = query_target.get(entity) else { continue };
                    for (entity, _center) in lookup.query_around(transform.translation(), *radius) {
                        let Ok(target_agent) = query_target.get_component::<Agent>(*entity) else { continue };
                        if target_agent.eq(agent) { continue; }
                        let Ok(mut integrity) = query_unit.get_mut(*entity) else { continue };
                        integrity.apply_damage(*damage);
                    }
                }
            }
        }
        commands.entity(entity).remove::<ImpactEffect>();
        if trajectory.is_none() { commands.entity(entity).despawn_recursive(); }
    }
}

pub fn resupply_military_phase(
    mut commands: Commands,
    query_unit: Query<(
        Entity, Option<&MilitarySupply>, Option<&MatterBinding>, Option<&UnderConstruction>, Option<&Suspended>,
        Option<&UpgradeAmplitude>, Option<&UpgradeFrequency>, Option<&UpgradeRange>,
    ), With<MilitaryBinding>>
){
    for (
        entity, supply, matter, construction, suspended,
        amplitude, frequency, range
    ) in query_unit.iter() {
        if supply.map_or(false, |supply|supply.snapshot) { continue; }
        let deficit = match matter {
            Some(MatterBinding::Consumption(consumption)) => !consumption.active(),
            Some(_) => true,
            None => false
        };
        let active = construction.is_none() && suspended.is_none() && !deficit;
        if active {
            let amplitude = amplitude.map_or(0,|upgrade|**upgrade);
            let frequency = frequency.map_or(0,|upgrade|**upgrade);
            let range = range.map_or(0,|upgrade|**upgrade);
            commands.entity(entity).insert(MilitarySupply { amplitude, frequency, range, snapshot: false });
        } else if supply.is_some() {
            commands.entity(entity).remove::<MilitarySupply>();
        }
    }
}

pub fn query_next_target<'a, T: ReadOnlyWorldQuery>(
    lookup: &SpatialLookupGrid<Entity>,
    agent: &Agent,
    transform: &GlobalTransform,
    query_target: &'a Query<(Entity, &Agent, &GlobalTransform), T>,
    min: f32, max: f32,
) -> Option<(Entity, &'a Agent, &'a GlobalTransform)> {
    let radius = max;
    let min = min * min; let max = max * max;
    let mut closest = None;
    let mut optimal: f32 = f32::MAX;
    for &(entity, position) in lookup.query_around(transform.translation(), radius) {
        let distance_squared = transform.translation().distance_squared(position);
        if distance_squared < min || distance_squared > max { continue; }
        let Ok(row) = query_target.get(entity) else { continue };
        if agent.eq(row.1) { continue; }
        if optimal > distance_squared {
            optimal = distance_squared;
            closest = Some(row);
        }
    }
    closest
}

pub fn update_military_targeting(
    lookup: Res<SpatialLookupGrid<Entity>>,
    time: Res<Time>,
    mut commands: Commands,
    mut events: EventWriter<CombatEvent>,
    mut query_unit: Query<(
        Entity, &Parent, &Agent, &mut MilitaryBinding, &MilitarySupply,
        Option<&TargetLock>, &GlobalTransform,
    )>,
    query_target: Query<(Entity, &Agent, &GlobalTransform), With<Integrity>>,
){
    for (
        entity, parent, agent, mut military, supply,
        target_lock, transform,
    ) in query_unit.iter_mut() {
        match military.as_mut() {
            MilitaryBinding::Trajectory {
                angular_limit, vertical_limit, radius,
                cooldown, cooldown_timer, orientation, damage, ..
            } => {
                if cooldown_timer.duration().is_zero() {
                    cooldown_timer.set_duration(Duration::from_secs_f32(*cooldown * supply.rate_multiplier()));
                }
                cooldown_timer.tick(time.delta());
                if !cooldown_timer.finished() { continue; }

                let target = target_lock
                    .and_then(|target|query_target.get(**target).ok())
                    .filter(|(_, _, transform)|{
                        let distance = transform.translation().distance(transform.translation());
                        radius.0 < distance && distance < radius.1 * supply.range_multipler()
                    })
                    .or_else(||query_next_target(&lookup, agent, transform, &query_target, radius.0, radius.1 * supply.range_multipler()));
                let Some((target_entity, _, target_transform)) = target else {
                    commands.entity(entity).remove::<TargetLock>();
                    continue;
                };
                if target_lock.map_or(true,|target|!target_entity.eq(target)) {
                    commands.entity(entity).insert(TargetLock(target_entity));
                }

                let target_position = target_transform.translation();
                let mut local_target_position = transform.compute_matrix().inverse().transform_point3(target_position);
                local_target_position.y = local_target_position.y.max(*vertical_limit);

                let forward = local_target_position.normalize();
                let right = forward.cross(Vec3::Y);
                let next_rotation = Quat::from_mat3(&Mat3::from_cols(
                    forward, right.cross(forward), right,
                )).normalize();

                let angle = Quat::angle_between(*orientation, next_rotation);
                if angle >= f32::EPSILON {
                    let fraction = angle.min(*angular_limit * time.delta_seconds()) / angle;
                    *orientation = Quat::slerp(*orientation, next_rotation, fraction);
                } else {
                    cooldown_timer.reset();
                    cooldown_timer.set_duration(Duration::from_secs_f32(*cooldown * supply.rate_multiplier()));
                    let distance = target_position.distance(transform.translation());
                    let projectile_speed = 0.25;
                    let effect = commands.spawn((
                        SpatialBundle::from_transform(Transform::from_matrix(transform.compute_matrix())),
                        agent.clone(),
                        TargetLock(target_entity),
                        TrajectoryEffect{
                            linked: false,
                            intro: Timer::from_seconds(distance * projectile_speed, TimerMode::Once),
                            outro: Timer::from_seconds(1.0 * projectile_speed, TimerMode::Once),
                        },
                        ImpactEffect::Single { interval: Timer::default(), damage: *damage + supply.amplitude }
                    )).id();
                    events.send(CombatEvent::ProjectileLaunch(effect, entity, target_entity));
                }
            },
            MilitaryBinding::Connection {
                damage, rate, radius, limit, released, degrade
            } => {
                if released >= limit { continue; }

                let Some(
                    (target_entity, _target_agent, _target_transform)
                ) = query_next_target(&lookup, agent, transform, &query_target, radius.0, radius.1 * supply.range_multipler()) else { continue };

                *released += 1;
                let limb = commands.spawn((
                    SpatialBundle::from_transform(Transform::from_matrix(transform.compute_matrix())),
                    agent.clone(),
                    TargetLock(target_entity),
                    SourceLink(entity),
                    TrajectoryEffect {
                        linked: true,
                        intro: Timer::from_seconds(1.8, TimerMode::Once),
                        outro: Timer::from_seconds(0.6, TimerMode::Once),
                    },
                    ImpactEffect::Single {
                        interval: Timer::from_seconds(*rate * supply.rate_multiplier(), TimerMode::Repeating),
                        damage: *damage + supply.amplitude
                    }
                )).id();
                if let Some(effect) = degrade {
                    effect.apply(commands.entity(limb));
                }
            },
            MilitaryBinding::Impact { radius, area, damage } => {
                let Some((target_entity, _, target_transform)) = target_lock
                    .and_then(|target|query_target.get(**target).ok()) else { continue };

                let distance_squared = target_transform.translation().distance_squared(transform.translation());
                if distance_squared > radius.0 * radius.0 { continue; }
                let effect = commands.spawn((
                    SpatialBundle::from_transform(Transform::from_matrix(transform.compute_matrix())),
                    agent.clone(),
                    TargetLock(target_entity),
                    SourceLink(entity),
                    ImpactEffect::Area {
                        interval: Default::default(),
                        damage: *damage + supply.amplitude,
                        radius: *area + supply.range_multipler(),
                    }
                )).id();
                commands.entity(entity).remove::<MilitarySupply>().insert(SourceLink(effect));
            },
            _ => {}
        }
    }
}

use super::{MapGrid, GridTileIndex};
use crate::common::adjacency::breadth_first_search;
use crate::effects::animation::MovementFormation;

pub fn redirect_unit_directive(
    mut commands: Commands,
    lookup: Res<SpatialLookupGrid<Entity>>,
    query_grid: Query<(&MapGrid, &GlobalTransform)>,
    query_unit: Query<(
        Entity, &Parent, &Agent, &GridTileIndex, &GlobalTransform, Option<&FollowingPath>,
        &MilitaryBinding, &MilitarySupply, Option<&TargetLock>
    ), With<MovementFormation>>,
    query_target: Query<(Entity, &Agent, &GlobalTransform), With<Integrity>>,
){
    for (
        entity, parent, agent, tile_index, transform, movement,
        military, supply, target_lock
    ) in query_unit.iter() {
        let radius = military.radius() * supply.range_multipler();
        let target_position = if let Some(movement) = movement {
            if !movement.stepped_over() || !military.is_close_range() { continue; }
            
            let target = query_next_target(&lookup, agent, transform, &query_target, 0.0, radius);

            if target_lock.is_some() && target.is_none() { commands.entity(entity).remove::<TargetLock>(); }
            let Some((target_entity, _, target_transform)) = target else { continue };
            
            if target_lock.map_or(true,|target|!target_entity.eq(target)) {
                commands.entity(entity).insert(TargetLock(target_entity));
            }
            target_transform.translation()
        } else {
            if !military.is_close_range() && target_lock.is_some() { continue; }
            let mut closest = None;
            let mut optimal: f32 = f32::MAX;
            for (target_entity, target_agent, target_transform) in query_target.iter() {
                if agent.eq(target_agent) { continue; }
                let distance_squared = transform.translation().distance_squared(target_transform.translation());
                if optimal > distance_squared {
                    optimal = distance_squared;
                    closest = Some((target_entity, target_transform.translation()));
                }
            }
            let Some((target_entity, target_position)) = closest else { continue };
            if optimal <= radius * radius && military.is_close_range() {
                if target_lock.map_or(true,|target|!target_entity.eq(target)) {
                    commands.entity(entity).insert(TargetLock(target_entity));
                }
            }
            target_position
        };
        
        let Ok((grid, parent_transform)) = query_grid.get(parent.get()) else { continue };
        let local_target_position = parent_transform.compute_matrix().inverse().transform_point3(target_position);

        let nodes = breadth_first_search(
            &tile_index.0,
            |&index|grid.graph.neighbors(index).unwrap().iter()
            .filter(|&i|
                grid.tiles[*i].is_empty() || grid.tiles[*i].reference.is_some()
            ),
            |index|
            Some(FloatOrd(grid.tiles[*index].transform.translation.distance_squared(local_target_position)))
        );
        if let Some(movement) = movement {
            let split = (movement.prev() + 1).min(movement.path.len() - 1);
            let mut path = movement.path.clone();
            assert_eq!(path[split], nodes[0]);
            if path[path.len() - 1] != nodes[nodes.len() - 1] && split - 1 + nodes.len() > 1 {
                path.truncate(split);
                path.extend_from_slice(&nodes);
                commands.entity(entity).insert(FollowingPath { path: nodes, ..movement.clone() });
            }
        } else if nodes.len() > 1 {
            commands.entity(entity).insert(FollowingPath::from(nodes));
        }
    }
}

pub fn apply_degradation_effect(
    mut commands: Commands,
    lookup: Res<SpatialLookupGrid<Entity>>,
    query_unit: Query<(Entity, &Agent, &MilitaryBinding, &MilitarySupply, &GlobalTransform)>,
    query_target: Query<(Entity, &Agent, &GlobalTransform), With<Integrity>>,
){
    for (
        entity, agent, military, supply, transform,
    ) in query_unit.iter() {
        let MilitaryBinding::Area { radius, degrade } = military else { continue };
          
        let center = transform.translation();
        let min = radius.0 * radius.0;
        let radius = radius.1 * supply.range_multipler();
        let max = radius * radius;
        
        for &(entity, position) in lookup.query_around(center, radius) {
            let distance_squared = position.distance_squared(center);
            if distance_squared < min || distance_squared > max { continue; }
            let Ok((_, target_agent, _)) = query_target.get(entity) else { continue };
            if agent.eq(target_agent) { continue; }

            if let Some(effect) = degrade {
                effect.apply(commands.entity(entity));
            }
        }
    }
}

pub fn propagate_degradation_effect(
    mut commands: Commands,
    query: Query<(&TargetLock, &ImpactEffect, &DegradeImmobilize, Option<&TrajectoryEffect>)>
){
    for (target, impact, immobilize, trajectory) in query.iter() {
        if trajectory.map_or(false, |trajectory|!trajectory.intro.finished()) { continue; }
        commands.entity(**target).insert_add(immobilize.clone());
    }
}