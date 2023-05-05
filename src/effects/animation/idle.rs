use bevy::prelude::*;
use bevy::scene::SceneInstance;
use crate::common::loader::AssetBundle;
use crate::scene::{ModelAssetBundle, UnitBlueprint};
use crate::logic::UnderConstruction;
use super::{UnitAnimation, AnimationState, AnimationEvent};

pub fn play_unit_animation(
    mut events: EventWriter<AnimationEvent>,
    model_bundle: Res<AssetBundle<ModelAssetBundle>>,
    query: Query<(&Parent, &Children, &UnitAnimation), With<SceneInstance>>,
    query_unit: Query<Ref<AnimationState>, (With<Handle<UnitBlueprint>>, Without<UnderConstruction>)>,
    animations: Res<Assets<AnimationClip>>,
    mut query_animation: Query<&mut AnimationPlayer>,
){
    for (parent, children, animation) in query.iter() {
        let Ok(state) = query_unit.get(parent.get()) else { continue };
        if !state.is_changed() { continue; }
        match animation {
            UnitAnimation::Idle(label) => {
                let handle = model_bundle.animations.get(label).unwrap();
                let paused = match state.as_ref() {
                    AnimationState::Active | AnimationState::Enabled => false,
                    _ => true
                };
                for &entity in children.iter() {
                    let Ok(mut player) = query_animation.get_mut(entity) else { continue };
                    if paused {
                        player.pause();
                    } else {
                        player.resume();
                        player.play(handle.clone_weak()).repeat();
                    }
                }
            },
            UnitAnimation::Trigger(label) => {
                let handle = model_bundle.animations.get(label).unwrap();
                let animation = animations.get(handle).unwrap();
                let reverse: bool = match state.as_ref() {
                    AnimationState::Active => false,
                    _ => true,
                };
                for &entity in children.iter() {
                    let Ok(mut player) = query_animation.get_mut(entity) else { continue };
                    let elapsed = player.elapsed();
                    if reverse && elapsed > 0.0 {
                        events.send(AnimationEvent::Trigger(parent.get(), reverse, elapsed));
                        player.play(handle.clone_weak())
                            .set_elapsed(elapsed.min(animation.duration()) - animation.duration()).set_speed(-1.0);
                    } else if !reverse && elapsed == 0.0 {
                        events.send(AnimationEvent::Trigger(parent.get(), reverse, 0.0));
                        player.play(handle.clone_weak()).set_speed(1.0);
                    } else if !reverse && elapsed <= 0.0 {
                        let elapsed = (elapsed + animation.duration()).max(0.0);
                        events.send(AnimationEvent::Trigger(parent.get(), reverse, elapsed));
                        player.play(handle.clone_weak())
                            .set_elapsed(elapsed).set_speed(1.0);
                    }
                }
            },
            _ => {}
        }
    }
}