use bevy::prelude::*;

#[derive(Clone, PartialEq)]
pub enum ConstructionEvent {
    Begin { entity: Entity, parent: Entity, index: usize, extend: bool },
    Assemble { entity: Entity, parent: Entity, index: usize, extend: bool },
    Dismantle { entity: Entity, parent: Entity, index: usize },
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CombatEvent {
    Destruct(Entity),
    Hit(Entity),
    ProjectileLaunch(Entity, Entity, Entity),
    ProjectileHit(Entity, Entity),
}