use bevy::prelude::*;
use crate::scene::UnitBlueprint;
use crate::logic::Agent;
use super::{ViewMode, ActionSelector};

#[derive(Component, Deref, DerefMut, Clone)]
pub struct EventTrigger<T>(pub T);

#[derive(Clone, PartialEq)]
pub enum InteractionEvent {
    Construct(Agent, Entity, usize, Handle<UnitBlueprint>),
    Toggle(Entity),
    Deconstruct(Entity),
    EnterMode(Option<ViewMode>),
    Execute(Entity, ActionSelector, u8),
}