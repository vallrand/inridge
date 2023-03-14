use bevy::prelude::*;
use crate::common::animation::{Animator,Track,AnimationStateMachine,StateMachineTransition};
use crate::common::loader::AssetBundle;
use crate::materials::{ColorUniform, UnlitMaterial};
use crate::scene::bundles::effects::EffectAssetBundle;
use super::border::BorderOutline;
use crate::logic::{MapGrid,NetworkGroupList};
use crate::interaction::{GridSelection, SelectionState};

pub fn update_grid_tile_highlight(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<UnlitMaterial>>,
    effect_bundle: Res<AssetBundle<EffectAssetBundle>>,
    query_grid: Query<
        (Entity, &MapGrid, &GridSelection),
        Changed<GridSelection>
    >,
    mut previous_selection: Local<Option<Entity>>,
){
    let Ok(
        (parent, grid, selection)
    ) = query_grid.get_single() else { return };

    if let Some(entity) = previous_selection.take() {
        commands.entity(entity).insert(SelectionState::None);
    }

    let hover = commands.spawn((
        SpatialBundle::default(),
        meshes.add(BorderOutline::from_single(&grid, selection.0)
        .with_stroke(0.04, 0.5, false).with_offset(0.05).into()),
        materials.add(UnlitMaterial{
            color: Color::rgb(0.4,0.8,0.6),
            diffuse: Some(effect_bundle.border.clone()),
            depth_bias: 1000, ..Default::default()
        }),
        ColorUniform::from(Color::NONE),
        AnimationStateMachine::new(vec![
            StateMachineTransition::new(SelectionState::None, SelectionState::Hover, 0.2),
            StateMachineTransition::new(SelectionState::Hover, SelectionState::None, 0.2),
        ], SelectionState::None),
        SelectionState::Hover,
        Animator::<ColorUniform>::new()
            .add(Track::from_static(Color::NONE.into()).with_state(SelectionState::None))
            .add(Track::from_static(Color::WHITE.into()).with_state(SelectionState::Hover)),
        bevy::pbr::NotShadowCaster,
        bevy::pbr::NotShadowReceiver,
    )).id();
    commands.entity(parent).add_child(hover);
    previous_selection.replace(hover);
}

pub fn update_grid_group_highlight(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<UnlitMaterial>>,
    effect_bundle: Res<AssetBundle<EffectAssetBundle>>,
    query_grid: Query<
        (Entity, &MapGrid, &GridSelection, &NetworkGroupList),
        Or<(Changed<GridSelection>, Changed<NetworkGroupList>)>
    >,
    mut previous_selection: Local<Option<(Entity, u64)>>,
){
    let Ok(
        (parent, grid, selection, groups)
    ) = query_grid.get_single() else { return };

    let group = grid.visited.get(&selection.0).map(|&group_index|
        groups[group_index].list.iter().map(|(i,_)|*i).collect::<Vec<usize>>()
    );
    let hash = group.as_ref().map(|list|{
        list.iter().fold(list.len(), |sum, i| sum + i) * list.len() + selection.0
    }).unwrap_or_default() as u64;
    if previous_selection.map(|(_,prev_hash)|prev_hash == hash).unwrap_or(false) { return; }

    if let Some((entity,_)) = previous_selection.take() {
        commands.entity(entity).insert(SelectionState::None);
    }

    let Some(group) = group else { return };
    let border = commands.spawn((
        SpatialBundle::default(),
        meshes.add(BorderOutline::from_group(&grid, &group, true)
        .with_stroke(0.02, 0.5, false).with_offset(0.02).into()),
        materials.add(UnlitMaterial{
            color: Color::rgb(0.8,0.8,0.2),
            diffuse: Some(effect_bundle.border.clone()),
            depth_bias: 200, ..Default::default()
        }),
        ColorUniform::from(Color::NONE),
        AnimationStateMachine::new(vec![
            StateMachineTransition::new(SelectionState::None, SelectionState::Hover, 0.2),
            StateMachineTransition::new(SelectionState::Hover, SelectionState::None, 0.2),
        ], SelectionState::None),
        SelectionState::Hover,
        Animator::<ColorUniform>::new()
            .add(Track::from_static(Color::NONE.into()).with_state(SelectionState::None))
            .add(Track::from_static(Color::WHITE.into()).with_state(SelectionState::Hover)),
        bevy::pbr::NotShadowCaster,
        bevy::pbr::NotShadowReceiver,
    )).id();
    commands.entity(parent).add_child(border);
    previous_selection.replace((border, hash));
}