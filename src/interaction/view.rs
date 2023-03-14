use bevy::prelude::*;
use crate::logic::Agent;
use crate::scene::GlobalState;
use super::path::ActionPath;

#[derive(Resource, Clone, PartialEq, Default)]
pub enum ViewMode {
    #[default] Menu,
    Default(Agent),
    Action(Entity, usize, ActionSelector)
}
impl From<&ViewMode> for GlobalState {
    fn from(value: &ViewMode) -> Self { match value {
        ViewMode::Menu => GlobalState::Menu,
        ViewMode::Default(_) => GlobalState::Running,
        _ => GlobalState::Paused,
    } }
}

#[derive(Clone, PartialEq)]
pub enum ActionSelector {
    FollowPath(Option<ActionPath>),
    Target(Option<ActionPath>),
}