mod view;
pub use view::*;
mod event;
pub use event::*;
mod action;
pub use action::*;
mod selection;
pub use selection::*;
mod transition;
pub use transition::*;

pub mod path;

use bevy::prelude::*;
use crate::common::animation::AnimationSet;
pub struct InteractionPlugin; impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ViewMode>();
        app.add_event::<InteractionEvent>();

        app.add_system(transition::schedule_state_transitions::<transition::SelectionState>
            .before(AnimationSet::StateMachine));

        app.add_system(action::process_interaction_event
            .in_base_set(CoreSet::PreUpdate)
            .before(bevy::scene::scene_spawner)
            .after(crate::interface::dispatch_interaction_events)
        );

        app.add_system(path::select_action_path
            .after(action::process_interaction_event)
            .in_base_set(CoreSet::PreUpdate));
        
        
        app.add_system(selection::update_grid_selection.in_base_set(CoreSet::PreUpdate));
    }
}