use bevy::prelude::*;
use crate::common::loader::AssetBundle;
use crate::common::animation::{Animator, Track, TransformRotation};
use crate::interaction::{ViewMode, GridSelection, InteractionEvent, SelectionState};
use crate::logic::{Agent, GroupLink, Suspended};
use crate::scene::InterfaceAssetBundle;
use super::layout::OverlayLayout;
use super::shared::{ControlComponent, ControlComponentDescriptor};

pub fn update_unit_controls_toggle(
    mut commands: Commands,
    mut component: Local<Option<ControlComponent>>,
    mode: Res<ViewMode>,
    layout: Res<OverlayLayout>,
    interface_bundle: Res<AssetBundle<InterfaceAssetBundle>>,
    query_unit: Query<(Entity, &Agent, Option<&Suspended>), (With<GroupLink>, With<GridSelection>)>,
){
    let component = component.get_or_insert_with(||{
        let component = ControlComponent::new(&mut commands, &layout, ControlComponentDescriptor {
            quadrant: 0, size: 24.0, angle: std::f32::consts::PI * 2.0 / 8.0,
            image_panel: interface_bundle.panel_single.clone(),
            image_icon: interface_bundle.icon_toggle.clone(),
            color_enabled: interface_bundle.color_enabled,
            color_active: Some(interface_bundle.color_disabled),
            ..Default::default()
        });
        commands.entity(component.icon).insert(
        Animator::<TransformRotation>::new()
                .add(Track::from_static(Quat::from_rotation_z(std::f32::consts::PI / 2.0).into()).with_state(SelectionState::Active))
                .add(Track::from_static(Quat::IDENTITY.into()).with_state(SelectionState::Enabled))
        );
        component
    });
    if let ViewMode::Default(global_agent) = mode.as_ref() {
        if let Ok((entity, agent, suspended)) = query_unit.get_single() {
            component.set_state(&mut commands, match suspended {
                None => SelectionState::Enabled,
                Some(_) => SelectionState::Active
            });
            if global_agent == agent {
                component.set_trigger(&mut commands, InteractionEvent::Toggle(entity));
            } else {
                component.clear_trigger(&mut commands);
            }
            return;
        }
    }
    component.clear_trigger(&mut commands);
    component.set_state(&mut commands, SelectionState::Disabled);
}