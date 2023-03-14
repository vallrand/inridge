use bevy::prelude::*;
use crate::common::animation::label::Label;
use crate::common::animation::AnimationStateMachine;

#[derive(Clone, PartialEq, Eq)]
pub enum StateTrigger {
    None,
    Remove
}

#[derive(Component, Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum SelectionState {
    None,
    Enabled,
    Disabled,
    Hover,
    Active
}

impl From<SelectionState> for StateTrigger {
    fn from(value: SelectionState) -> Self { match value {
        SelectionState::None => StateTrigger::Remove,
        _ => StateTrigger::None
    } }
}

pub fn schedule_state_transitions<T: Label + Component + Clone + PartialEq + Into<StateTrigger>>(
    mut commands: Commands,
    mut query: Query<(Entity, &T, &mut AnimationStateMachine)>
){
    for (entity, selection_state, mut state_machine) in query.iter_mut() {
        let Some(current_state) = state_machine.current_state::<T>() else { continue };
        let current_state = current_state.clone();
        state_machine.trigger(Some(&current_state), selection_state);
        if state_machine.is_idle() && current_state.eq(selection_state) && Into::<StateTrigger>::into(current_state) == StateTrigger::Remove {
            commands.entity(entity).despawn_recursive();
        }
    }
}