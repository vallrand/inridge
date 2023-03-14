use bevy::prelude::*;
use crate::common::loader::AssetBundle;
use crate::interaction::{ViewMode, GridSelection, InteractionEvent, SelectionState, ActionSelector};
use crate::logic::{Agent, MapGrid, GroupLink, UnderConstruction, Suspended, GridTileIndex, MatterBinding, UnitFabrication};
use crate::scene::{InterfaceAssetBundle, UnitBlueprint};
use super::layout::OverlayLayout;
use super::shared::{ControlComponent, ControlComponentDescriptor};

pub fn update_unit_controls_action(
    mut commands: Commands,
    mut component: Local<Option<ControlComponent>>,
    mut events: EventWriter<InteractionEvent>,
    mode: Res<ViewMode>,
    layout: Res<OverlayLayout>,
    interface_bundle: Res<AssetBundle<InterfaceAssetBundle>>,
    blueprints: Res<Assets<UnitBlueprint>>,
    mut query_unit: ParamSet<(
        Query<(
            Entity, &Agent, &GridTileIndex, &Handle<UnitBlueprint>, Option<&MatterBinding>,
        ), (With<GridSelection>, With<GroupLink>, Without<UnderConstruction>, Without<Suspended>)>,
        Query<&Handle<UnitBlueprint>, With<GroupLink>>
    )>
){
    let component = component.get_or_insert_with(||ControlComponent::new(
        &mut commands, &layout, ControlComponentDescriptor {
            quadrant: 0, size: 20.0, angle: std::f32::consts::PI * 1.0 / 8.0,
            image_panel: interface_bundle.panel_single.clone(),
            image_icon: interface_bundle.icon_action.clone(),
            color_enabled: interface_bundle.color_enabled,
            color_active: Some(interface_bundle.color_active),
            ..Default::default()
        }
    ));
    match mode.as_ref() {
        ViewMode::Default(global_agent) => {
            let query_unit = query_unit.p0();
            if let Some(trigger) = query_unit.get_single().ok()
            .and_then(|(entity, agent, tile_index, blueprint_handle, matter)|
                if agent != global_agent { None }else if match matter.as_ref() {
                    Some(MatterBinding::Consumption(consumption)) => !consumption.active(),
                    _ => false
                } { None } else {
                    blueprints.get(blueprint_handle)
                    .and_then(|blueprint|blueprint.action.as_ref())
                    .map(|action|InteractionEvent::EnterMode(
                        Some(ViewMode::Action(entity, tile_index.0, action.clone().into()))
                    ))
                }
            ) {
                component.set_state(&mut commands, SelectionState::Enabled);
                component.set_trigger(&mut commands, trigger);
                return;
            }
        },
        ViewMode::Action(entity, _, selector) => {
            let query_unit = query_unit.p1();
            if let Some(_) = query_unit.get(*entity).ok().and_then(|handle|blueprints.get(handle))
            .and_then(|blueprint|blueprint.action.as_ref()) {
                component.set_state(&mut commands, SelectionState::Active);
                component.set_trigger(&mut commands, InteractionEvent::Execute(*entity, selector.clone(), UnitFabrication::MILITARY));
                return;
            } else {
                events.send(InteractionEvent::EnterMode(None));
            }
        },
        ViewMode::Menu => {}
    }
    component.clear_trigger(&mut commands);
    component.set_state(&mut commands, SelectionState::Disabled);
}

pub fn update_unit_subcontrols_action(
    mut commands: Commands,
    mut component: Local<Option<ControlComponent>>,
    mode: Res<ViewMode>,
    layout: Res<OverlayLayout>,
    interface_bundle: Res<AssetBundle<InterfaceAssetBundle>>,
    query_unit: Query<&UnitFabrication, With<GroupLink>>,
    query_grid: Query<&MapGrid, With<GridSelection>>
){
    let offset: i32 = 0;
    let component = component.get_or_insert_with(||ControlComponent::new(
        &mut commands, &layout, ControlComponentDescriptor {
            quadrant: 1, size: 16.0, angle: -((offset as f32 + 0.5) / 5.0).asin(),
            image_panel: interface_bundle.panel_single.clone(),
            image_icon: interface_bundle.icon_store.clone(),
            color_enabled: interface_bundle.color_enabled,
            text_style: Some(interface_bundle.text_style_secondary.clone()),
            ..Default::default()
        }
    ));
    match mode.as_ref() {
        ViewMode::Action(entity, _index, selector) => {
            if let Ok(fabrication) = query_unit.get(*entity) {
                let empty = if let ActionSelector::Target(Some(path)) = selector {
                    path.last()
                        .map_or(false,|i|query_grid.get_single()
                            .map_or(false,|grid|grid.tiles[*i].is_empty()))
                } else { false };

                if fabrication.group != UnitFabrication::MILITARY && empty {
                    component.set_state(&mut commands, SelectionState::Enabled);
                    component.set_label(&mut commands, fabrication.key.clone());
                    component.set_trigger(&mut commands, InteractionEvent::Execute(*entity, selector.clone(), fabrication.group));
                    return;
                }
            }
        },
        _ => {}
    }
    component.clear_trigger(&mut commands);
    component.set_state(&mut commands, SelectionState::Disabled);
}