use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use crate::common::loader::AssetBundle;
use super::AudioAssetBundle;
use super::GlobalState;

pub fn update_theme(
    audio_bundle: Res<AssetBundle<AudioAssetBundle>>,
    audio: Res<Audio>,
    keycode: Res<Input<KeyCode>>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    mut prev: Local<Option<Handle<AudioInstance>>>,
    state: Res<State<GlobalState>>,
    mut mute: Local<bool>,
){
    if keycode.just_pressed(KeyCode::M) {
        let mute_toggle = *mute;
        *mute = !mute_toggle;
    }
    let should_play = !*mute && match state.0 {
        GlobalState::Menu => false,
        _ => true
    };
    if should_play && prev.is_none() {
        prev.replace(audio.play(audio_bundle.theme.clone())
        .looped()
        .with_volume(0.4)
        .handle());
    } else if !should_play {
        if let Some(audio_instance) = prev.take().and_then(|handle|audio_instances.get_mut(&handle)) {
            audio_instance.stop(AudioTween::linear(std::time::Duration::from_secs_f32(2.0)));
        }
    }
}