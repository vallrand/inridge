pub mod bundles;
mod stage;
mod theme;
mod setup;
mod camera;
mod input;
mod lighting;

pub use bundles::blueprint::*;
pub use bundles::environment::*;
pub use bundles::interface::*;
pub use bundles::effects::*;
pub use bundles::models::*;
pub use bundles::audio::*;

use bevy::prelude::*;
use crate::common::loader::{LoadingState, AssetBundle, RonAssetPlugin};

#[derive(States, Clone, PartialEq, Eq, Hash, Default, Debug)]
pub enum GlobalState {
    #[default] Menu,
    Running,
    Paused,
}

pub struct DemoPlugin; impl Plugin for DemoPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(RonAssetPlugin::<StageBlueprint>::new("stage.ron"));
        app.add_plugin(RonAssetPlugin::<UnitBlueprint>::new("unit.ron"));

        app.add_plugin(input::InputManagerPlugin);

        app.init_resource::<AssetBundle<BlueprintAssetBundle>>();
        app.init_resource::<AssetBundle<EnvironmentAssetBundle>>();
        app.init_resource::<AssetBundle<EffectAssetBundle>>();
        app.init_resource::<AssetBundle<InterfaceAssetBundle>>();
        app.init_resource::<AssetBundle<ModelAssetBundle>>();
        app.init_resource::<AssetBundle<AudioAssetBundle>>();

        app.add_system(stage::load_stage.in_schedule(OnExit(GlobalState::Menu)));
        app.add_systems((
            apply_system_buffers, stage::unload_stage, apply_system_buffers,
        ).chain().in_schedule(OnEnter(GlobalState::Menu)));
        app.add_system(theme::update_theme.in_set(OnUpdate(LoadingState::Running)));
        app.add_system(lighting::update_orbiting_transforms.in_set(OnUpdate(GlobalState::Running)));
        app.add_startup_system(setup::setup_scene);

        app.add_system(camera::update_camera_view.in_base_set(CoreSet::PreUpdate).after(input::handle_input_system));
    }
}