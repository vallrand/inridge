use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use crate::common::loader::{AssetBundleList, ScopedAssetServer};

pub struct AudioAssetBundle {
    pub transition: std::time::Duration,
    pub theme_ambience: Handle<AudioSource>,
    pub theme_conflict: Handle<AudioSource>,
    pub open_gate: Handle<AudioSource>,
    pub vessel_deploy: Handle<AudioSource>,
    pub walker: Handle<AudioSource>,
    pub disable: Handle<AudioSource>,
    pub enable: Handle<AudioSource>,
    pub explosion_fire: Handle<AudioSource>,
    pub explosion_acid: Handle<AudioSource>,
    pub hit_impact: Handle<AudioSource>,
    pub launch_projectile: Handle<AudioSource>,
    pub locust_jump: Handle<AudioSource>,
    pub construct: Handle<AudioSource>,
    pub slither: Handle<AudioSource>,
    pub pulsar: Handle<AudioSource>,
    pub select: Handle<AudioSource>,
}
impl AssetBundleList for AudioAssetBundle {
    fn from_asset_server(asset_server: &ScopedAssetServer) -> Self { Self {
        transition: std::time::Duration::from_secs_f32(30.0),
        theme_ambience: asset_server.load("sounds/theme_insector.mp3"),
        theme_conflict: asset_server.load("sounds/theme_recline.mp3"),
        open_gate: asset_server.load("sounds/gate_open.mp3"),
        vessel_deploy: asset_server.load("sounds/deploy.mp3"),
        walker: asset_server.load("sounds/walk.mp3"),
        disable: asset_server.load("sounds/disable.mp3"),
        enable: asset_server.load("sounds/enable.mp3"),
        explosion_fire: asset_server.load("sounds/explosion_fire.mp3"),
        explosion_acid: asset_server.load("sounds/explosion_acid.mp3"),
        hit_impact: asset_server.load("sounds/hit.mp3"),
        launch_projectile: asset_server.load("sounds/launch.mp3"),
        locust_jump: asset_server.load("sounds/jump.mp3"),
        construct: asset_server.load("sounds/construct.mp3"),
        slither: asset_server.load("sounds/slither.mp3"),
        pulsar: asset_server.load("sounds/pulsar.mp3"),
        select: asset_server.load("sounds/select.mp3"),
    } }
}