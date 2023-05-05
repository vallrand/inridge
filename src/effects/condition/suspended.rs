use bevy::prelude::*;
use crate::effects::animation::{AnimationSettings, AnimationState};
use crate::materials::{ColorUniform, ModelEffectLayeredMaterial};
use crate::logic::GroupLink;

pub fn animate_unit_condition_suspended(
    time: Res<Time>,
    settings: Res<AnimationSettings>,
    query_unit: Query<(Entity, Ref<AnimationState>), (With<GroupLink>, Or<(Changed<AnimationState>, Added<GroupLink>)>)>,
    children: Query<&Children>,
    mut query: Query<&mut ColorUniform, With<Handle<ModelEffectLayeredMaterial>>>,
    mut transitions: Local<Vec<(Entity, bool)>>
){
    let delta: f32 = time.delta_seconds() / settings.condition_transition_duration;
    transitions.retain_mut(|(entity, reverse)|{
        let mut done: bool = false;
        for entity in children.iter_descendants(*entity) {
            let Ok(mut uniform) = query.get_mut(entity) else { continue };
            let (value, max) = if *reverse {
                ((uniform.color.r() + delta).min(1.0), 1.0)
            } else {
                ((uniform.color.r() - delta).max(0.0), 0.0)
            };
            done = done || value == max;
            uniform.color.set_r(value);
        }
        !done
    });
    for (entity, state) in query_unit.iter() {
        let disabled = match state.as_ref() {
            AnimationState::Deficit | AnimationState::Disabled => true,
            _ => false
        };
        if state.is_added() && !disabled { continue; }
        transitions.retain(|(prev_entity, _reverse)|!entity.eq(prev_entity));
        transitions.push((entity, disabled));
    }
}