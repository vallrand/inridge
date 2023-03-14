pub mod label;
pub mod ease;
pub mod animator;
mod state;
mod transform;

pub use animator::*;
pub use state::*;
pub use transform::*;

use bevy::prelude::*;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, SystemSet)]
pub enum AnimationSet {
    StateMachine,
    Timeline,
    Animator
}

#[derive(Clone, PartialEq)]
pub enum AnimationEvent {
    AnimationEnd,
    TransitionEnd(Entity),
    Trigger(&'static str)
}

pub struct AnimationPlugin; impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<AnimationEvent>();
        app.configure_sets((
            AnimationSet::StateMachine,
            AnimationSet::Timeline,
            AnimationSet::Animator,
        ).chain().in_base_set(CoreSet::Update));

        app.add_system(update_state_machine.in_set(AnimationSet::StateMachine));
        app.add_system(update_timeline.in_set(AnimationSet::Timeline));

        app.add_system(animate_component::<BackgroundColor>.in_set(AnimationSet::Animator));
        app.add_system(animate_component::<TransformRotation>.in_set(AnimationSet::Animator));
        app.add_system(animate_component::<TransformTranslation>.in_set(AnimationSet::Animator));
        app.add_system(animate_component::<TransformScale>.in_set(AnimationSet::Animator));
    }
}