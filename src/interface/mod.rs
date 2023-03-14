mod trigger;
pub use trigger::*;

pub mod layout;
pub mod shared;
pub mod construct;
pub mod deconstruct;
pub mod toggle;
pub mod control;
pub mod indicator;

use bevy::prelude::*;
use crate::common::loader::LoadingState;
use crate::logic::LogicSet;

pub struct InterfacePlugin; impl Plugin for InterfacePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<layout::OverlayLayout>();
        app.add_startup_system(layout::setup_interface_view.in_base_set(StartupSet::Startup));

        app.add_system(trigger::dispatch_interaction_events
            .after(bevy::ui::UiSystem::Focus)
            .run_if(in_state(LoadingState::Running))
            .in_base_set(CoreSet::PreUpdate));

        app.add_systems((
            deconstruct::update_unit_controls_deconstruct,
            toggle::update_unit_controls_toggle,
            construct::update_construction_menu,
            control::update_unit_controls_action,
            control::update_unit_subcontrols_action,
            indicator::update_indicator_display,
        ).after(LogicSet::PostUpdate)
        .in_set(OnUpdate(LoadingState::Running)));
    }
}