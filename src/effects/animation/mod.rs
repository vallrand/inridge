mod formation;
pub use formation::*;
mod extract;
pub use extract::*;
mod idle;
pub use idle::*;
mod relocation;
pub use relocation::*;
mod movement;
pub use movement::*;

use bevy::prelude::*;

#[derive(Component, serde::Deserialize, Clone, Debug)]
pub enum UnitAnimation {
    Idle(String),
    Trigger(String),
    Movement(String),
    HexMovement(String, String, String),
}

#[derive(Resource)]
pub struct AnimationSettings {
    pub bridge_transition_duration: f32,
    pub condition_transition_duration: f32,
}
impl FromWorld for AnimationSettings {
    fn from_world(_world: &mut World) -> Self { Self {
        bridge_transition_duration: 1.0,
        condition_transition_duration: 0.5,
    } }
}