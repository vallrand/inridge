use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use crate::common::loader::AssetBundle;
use crate::scene::{UnitBlueprint, AudioAssetBundle};
use crate::interaction::{EventTrigger, InteractionEvent};
use super::animation::{AnimationEvent, MovementFormation, MovementVariant};
use crate::logic::{UnitDirective, FollowingPath};

pub fn cleanup_spatial_audio(
    mut commands: Commands,
    mut query: Query<(Entity, &mut AudioEmitter)>,
    audio: Res<Audio>,
){
    for (entity, mut emitter) in query.iter_mut() {
        emitter.instances.retain(|instance_handle|{
            match audio.state(instance_handle) {
                PlaybackState::Stopped => false,
                _ => true
            }
        });
        if emitter.instances.is_empty() { commands.entity(entity).remove::<AudioEmitter>(); }
    }
}

pub fn trigger_sound_effect(
    mut commands: Commands,
    mut animation_events: EventReader<AnimationEvent>,
    audio_bundle: Res<AssetBundle<AudioAssetBundle>>,
    audio: Res<Audio>,
    blueprints: Res<Assets<UnitBlueprint>>,
    query_unit: Query<(&Children, &Handle<UnitBlueprint>)>,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<Button>, With<EventTrigger<InteractionEvent>>)>,
){
    for interaction in interaction_query.iter() {
        let Interaction::Clicked = interaction else { continue };
        audio.play(audio_bundle.select.clone());
    }
    for event in animation_events.iter() {
        match event {
            &AnimationEvent::Trigger(entity, reverse, _elapsed) => {
                let Ok((children, handle)) = query_unit.get(entity) else { continue };
                let Some(blueprint) = blueprints.get(handle) else { continue };

                let audio_handle = match (blueprint.action.as_ref(), blueprint.military.as_ref()) {
                    (Some(UnitDirective::OpenGate), _) => audio_bundle.open_gate.clone(),
                    (_, Some(_)) => audio_bundle.slither.clone(),
                    _ => audio_bundle.vessel_deploy.clone(),
                };
                let mut play = audio.play(audio_handle);
                if reverse { play.reverse(); }
                commands.entity(*children.first().unwrap()).insert(AudioEmitter{instances: vec![play.handle()]});
            },
            &AnimationEvent::Toggle(entity, toggle) => {
                let audio_handle = if toggle {
                    audio_bundle.enable.clone()
                } else {
                    audio_bundle.disable.clone()
                };
                commands.entity(entity).insert(AudioEmitter{instances: vec![
                    audio.play(audio_handle).handle()
                ]});
            },
            _ => {

            }
        }
    }
}

pub fn movement_sound_effect(
    mut commands: Commands,
    audio: Res<Audio>,
    audio_bundle: Res<AssetBundle<AudioAssetBundle>>,
    query_unit: Query<(&Children, Ref<FollowingPath>, Option<&MovementFormation>), With<Handle<UnitBlueprint>>>,
){
    for (children, movement, formation) in query_unit.iter() {
        if let Some(formation) = formation {
            if let MovementVariant::Staggered { .. } = formation.variant {
                if formation.last_reset {
                    commands.entity(*children.first().unwrap()).insert(AudioEmitter{instances:vec![
                        audio.play(audio_bundle.locust_jump.clone()).with_volume(0.2).handle()
                    ]});
                }
            }
        } else {
            if movement.last_step == 1 && movement.is_changed() && !movement.has_ended() {
                commands.entity(*children.first().unwrap()).insert(AudioEmitter{instances:vec![
                    audio.play(audio_bundle.walker.clone()).with_volume(0.5).handle()
                ]});
            }
        }
    }
}