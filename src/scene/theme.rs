use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use crate::common::loader::AssetBundle;
use crate::logic::CombatEvent;
use super::AudioAssetBundle;
use super::GlobalState;

#[derive(Clone, PartialEq, Eq)]
pub enum ThemeMode {
    Ambient,
    Conflict
}

pub fn update_theme(
    time: Res<Time>,
    mut events: EventReader<CombatEvent>,
    audio_bundle: Res<AssetBundle<AudioAssetBundle>>,
    audio: Res<Audio>,
    keycode: Res<Input<KeyCode>>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    mut prev: Local<Option<(ThemeMode, Handle<AudioInstance>)>>,
    state: Res<State<GlobalState>>,
    mut timer: Local<Timer>,
    mut mute: Local<bool>,
){
    if keycode.just_pressed(KeyCode::M) {
        let mute_toggle = *mute;
        *mute = !mute_toggle;
    }
    timer.tick(time.delta());
    for event in events.iter() {
        let (CombatEvent::Hit(_) | CombatEvent::ProjectileHit(_,_)) = event else { continue };
        timer.reset();
        timer.set_duration(audio_bundle.transition);
    }
    let theme = if *mute || state.0 == GlobalState::Menu {
        None
    }else if timer.finished() {
        Some(ThemeMode::Ambient)
    } else {
        Some(ThemeMode::Conflict)
    };
    if match (theme.as_ref(), prev.as_ref()) {
        (None, None) => true,
        (Some(next), Some((prev, _))) => next == prev,
        (_, _) => false
    } { return; }
    if let Some(audio_instance) = prev.take()
    .and_then(|(_,handle)|audio_instances.get_mut(&handle)) {
        audio_instance.stop(AudioTween::linear(std::time::Duration::from_secs_f32(2.0)));
    }
    let Some(theme) = theme else { return };
    let audio_instance = audio.play(match theme {
        ThemeMode::Ambient => audio_bundle.theme_ambience.clone(),
        ThemeMode::Conflict => audio_bundle.theme_conflict.clone(),
    })
    .looped()
    .with_volume(0.4)
    .handle();
    prev.replace((theme, audio_instance));
}