use bevy::prelude::*;
use crate::common::loader::AssetBundle;
use crate::interaction::{ViewMode, GridSelection, InteractionEvent, SelectionState};
use crate::logic::{Agent, GroupLink, MapGrid, NetworkGroupList, UnderConstruction, Suspended};
use crate::scene::{InterfaceAssetBundle, BlueprintAssetBundle, UnitBlueprint};
use super::layout::OverlayLayout;
use super::shared::{ControlComponent, ControlComponentDescriptor};

pub fn validate_construction(
    blueprint: &UnitBlueprint, unit: Option<&UnitBlueprint>, agent: &Agent,
    grid: &MapGrid, groups: &NetworkGroupList, tile_index: usize
) -> bool {
    let selected = &grid.tiles[tile_index];
    match (&blueprint.predecessor, unit) {
        (None, None) => selected.is_empty() && grid.iter_adjacent_groups(tile_index)
        .any(|group|groups[*group].agent.eq(agent)),
        (Some(key), Some(blueprint)) => blueprint.key.eq(key) && selected.flags & MapGrid::OWNERSHIP != 0,
        _ => false
    }
}

pub fn update_construction_menu(
    mut commands: Commands,
    mut components: Local<Vec<ControlComponent>>,
    mode: Res<ViewMode>,
    layout: Res<OverlayLayout>,
    interface_bundle: Res<AssetBundle<InterfaceAssetBundle>>,
    blueprint_bundle: Res<AssetBundle<BlueprintAssetBundle>>,
    blueprints: Res<Assets<UnitBlueprint>>,
    query_grid: Query<(Entity, &MapGrid, &GridSelection, &NetworkGroupList), Or<(Changed<MapGrid>, Changed<GridSelection>)>>,
    query_unit: Query<(&Agent, &Handle<UnitBlueprint>), (With<GroupLink>, Without<Suspended>, Without<UnderConstruction>)>,
){
    let mut offset: usize = 0;
    if let ViewMode::Default(global_agent) = mode.as_ref() {
        let Ok((parent, grid, selection, groups)) = query_grid.get_single() else { return };
        let selected = &grid.tiles[selection.0];
        
        let unit: Option<&UnitBlueprint> = selected.reference
            .and_then(|entity|query_unit.get(entity).ok())
            .and_then(|(agent, handle)|if agent == global_agent {
                blueprints.get(handle)
            }else{ None });

        for handle in blueprint_bundle.unit_blueprints.iter() {
            let Some(option) = blueprints.get(handle) else { continue };
            if !validate_construction(
                option, unit, global_agent, grid, groups, selection.0
            ) { continue; }

            if offset >= components.len() {
                components.push(ControlComponent::new(&mut commands, &layout, ControlComponentDescriptor {
                    quadrant: 1, size: 16.0, angle: -((offset as f32 + 0.5) / 5.0).asin(),
                    image_panel: interface_bundle.panel_single.clone(),
                    image_icon: interface_bundle.icon_store.clone(),
                    color_enabled: interface_bundle.color_enabled,
                    text_style: Some(interface_bundle.text_style_secondary.clone()),
                    ..Default::default()
                }));
            }
            
            
            let component = &mut components[offset];
            component.set_trigger(&mut commands, InteractionEvent::Construct(
                global_agent.clone(), parent, selection.0, handle.clone()
            ));
            component.set_state(&mut commands, SelectionState::Enabled);
            component.set_label(&mut commands, option.key.clone());
            offset += 1;
        }
    }
    for i in offset..components.len() {
        components[i].clear_trigger(&mut commands);
        components[i].set_state(&mut commands, SelectionState::Disabled);
    }
}