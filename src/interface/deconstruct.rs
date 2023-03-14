use bevy::prelude::*;
use crate::common::loader::AssetBundle;
use crate::interaction::{ViewMode, GridSelection, InteractionEvent, SelectionState};
use crate::logic::{Agent, GroupLink};
use crate::scene::InterfaceAssetBundle;
use super::layout::OverlayLayout;
use super::shared::{ControlComponent, ControlComponentDescriptor};

pub fn update_unit_controls_deconstruct(
    mut commands: Commands,
    mut component: Local<Option<ControlComponent>>,
    mode: Res<ViewMode>,
    layout: Res<OverlayLayout>,
    interface_bundle: Res<AssetBundle<InterfaceAssetBundle>>,
    query_unit: Query<(Entity, &Agent), (With<GridSelection>, With<GroupLink>)>,
){
    let component = component.get_or_insert_with(||ControlComponent::new(
        &mut commands, &layout, ControlComponentDescriptor {
            quadrant: 0, size: 16.0, angle: std::f32::consts::PI * 3.0 / 8.0,
            image_panel: interface_bundle.panel_single.clone(),
            image_icon: interface_bundle.icon_remove.clone(),
            color_enabled: interface_bundle.color_enabled,
            ..Default::default()
        }
    ));
    if let ViewMode::Default(global_agent) = mode.as_ref() {
        if let Ok((entity, agent)) = query_unit.get_single() {
            if global_agent == agent {
                component.set_state(&mut commands, SelectionState::Enabled);
                component.set_trigger(&mut commands, InteractionEvent::Deconstruct(entity));
                return;
            }
        }
    }
    component.clear_trigger(&mut commands);
    component.set_state(&mut commands, SelectionState::Disabled);
}