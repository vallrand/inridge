#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(dead_code)]

mod common;
mod extensions;

mod logic;
mod interaction;
mod interface;
mod effects;
mod scene;
mod materials;

use bevy::prelude::*;
use bevy_hanabi::prelude::*;
use bevy_kira_audio::prelude::*;

fn main() {
    App::new()
    .insert_resource(Msaa::Off)
    .insert_resource(ClearColor(Color::DARK_GRAY))
    .add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "[Inridge]".to_string(), 
            resolution: (1280., 720.).into(),
            ..Default::default()
        }), ..Default::default()
    }))
    .add_plugin(HanabiPlugin)
    .add_plugin(AudioPlugin)

    .add_plugin(common::noise::NoiseShaderPlugin)
    .add_plugin(materials::MaterialEffectPlugin)
    .add_plugin(common::animation::AnimationPlugin)
    
    .add_plugin(common::spline::SplinePlugin)
    .add_plugin(common::rig::ManipulatorTransformPlugin)
    .add_plugin(common::loader::LoaderPlugin)
    .add_plugin(common::raycast::RaycastPlugin)

    .add_plugin(logic::LogicPlugin)
    .add_plugin(interaction::InteractionPlugin)
    .add_plugin(interface::InterfacePlugin)
    .add_plugin(effects::EffectsPlugin)
    .add_plugin(scene::DemoPlugin)
    .run();
}