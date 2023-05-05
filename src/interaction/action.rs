use bevy::prelude::*;
use crate::common::loader::AssetBundle;
use crate::logic::{Agent, MapGrid, GridTileIndex, GroupLink, PriorityOrder, GlobalEconomy};
use crate::logic::{Suspended, ConstructionEvent, Integrity, UnitDirective, FollowingPath, FabricationGate};
use crate::scene::{GlobalState, UnitBlueprint, ModelAssetBundle};
use super::{InteractionEvent, ViewMode, ActionSelector};

pub fn construct_unit(
    commands: &mut Commands,
    parent: Entity, grid: &MapGrid,
    model_bundle: &AssetBundle<ModelAssetBundle>,
    blueprints: &Assets<UnitBlueprint>,
    (blueprint_handle, agent, index): (Handle<UnitBlueprint>, Agent, usize),
) -> Entity {
    let selected = &grid.tiles[index];
    let blueprint = blueprints.get(&blueprint_handle).unwrap();
    
    let model = model_bundle.model_from(&blueprint, commands);
    let entity = commands.spawn((
        GridTileIndex(index),
        SpatialBundle::from_transform(selected.transform.clone().with_scale(selected.transform.scale * blueprint.scale)),
        agent.clone(),
        blueprint_handle.clone(),
        blueprint.integrity.clone()
    )).insert_children(0, &[model])
    .set_parent(parent).id();
    blueprint.apply(commands.entity(entity), false);
    entity
}

pub fn construct_structure(
    commands: &mut Commands,
    construction_events: &mut EventWriter<ConstructionEvent>,
    parent: Entity, grid: &mut MapGrid,
    model_bundle: &AssetBundle<ModelAssetBundle>,
    blueprints: &Assets<UnitBlueprint>,
    (blueprint_handle, agent, index, order): (Handle<UnitBlueprint>, Agent, usize, u64),
    immediate: bool
) -> Entity {
    let blueprint = blueprints.get(&blueprint_handle).unwrap();
    let selected = &mut grid.tiles[index];

    let entity = if let Some(selected) = selected.reference { selected }else{
        commands.spawn((
            GridTileIndex(index),
            PriorityOrder(order),
            GroupLink::default(),
            SpatialBundle::from_transform(selected.transform.clone()),
        )).set_parent(parent).id()
    };

    let model = model_bundle.model_from(&blueprint, commands);
    commands.entity(entity)
        .insert_children(0, &[model])
        .insert(agent.clone())
        .insert(blueprint_handle.clone());
    blueprint.apply(commands.entity(entity), !immediate);
    selected.reference = Some(entity);
    if immediate {
        construction_events.send(ConstructionEvent::Assemble { entity, parent, index, extend: selected.reference.is_some() });
        selected.flags |= MapGrid::BLOCKER | MapGrid::OWNERSHIP;
    } else {
        construction_events.send(ConstructionEvent::Begin { entity, parent, index, extend: selected.reference.is_some() });
        selected.flags |= MapGrid::BLOCKER;
    }
    entity
}

pub fn process_interaction_event(
    mut next_state: ResMut<NextState<GlobalState>>,
    mut exit: EventWriter<bevy::app::AppExit>,
    mut commands: Commands,
    mut global: ResMut<GlobalEconomy>,
    mut mode: ResMut<ViewMode>,
    mut previous_mode: Local<ViewMode>,
    mut construction_events: EventWriter<ConstructionEvent>,
    mut interaction_events: EventReader<InteractionEvent>,

    blueprints: Res<Assets<UnitBlueprint>>,
    model_bundle: Res<AssetBundle<ModelAssetBundle>>,

    mut query_grid: Query<&mut MapGrid>,
    mut query_unit: ParamSet<(
        Query<(&Parent, &mut PriorityOrder, Option<&Suspended>)>,
        Query<&mut Integrity>,
        Query<&Handle<UnitBlueprint>>,
    )>
){
    for event in interaction_events.iter() {
        match event {
            &InteractionEvent::Construct(agent, parent, index, ref blueprint_handle) => {
                let Ok(mut grid) = query_grid.get_mut(parent) else { continue };
                if let Some(previous) = grid.tiles[index].reference {
                    commands.entity(previous).remove::<FabricationGate>();
                }
                construct_structure(
                    &mut commands, &mut construction_events,
                    parent, &mut grid, &model_bundle, &blueprints,
                    (blueprint_handle.clone(), agent, index, global.next_priority()), false
                );
            },
            &InteractionEvent::Deconstruct(entity) => {
                let mut query_unit = query_unit.p1();
                let Ok(mut integrity) = query_unit.get_mut(entity) else { continue };
                integrity.apply_damage(i32::MAX);
            },
            &InteractionEvent::Toggle(entity) => {
                let mut query_unit = query_unit.p0();
                let Ok((parent, mut order, suspended)) = query_unit.get_mut(entity) else { continue };
                let Ok(mut grid) = query_grid.get_mut(parent.get()) else { continue };
                if suspended.is_some() {
                    order.0 = global.next_priority();
                    commands.entity(entity).remove::<Suspended>();
                    grid.set_changed();
                } else {
                    commands.entity(entity).insert(Suspended);
                }
            },
            InteractionEvent::Execute(entity, selector, flags) => {
                let Some(blueprint) = query_unit.p2().get(*entity).ok()
                    .and_then(|handle|blueprints.get(handle)) else { continue };
                match blueprint.action {
                    None => continue,
                    Some(UnitDirective::Relocate) => {
                        let ActionSelector::FollowPath(path) = selector else { continue };
                        if let Some(path) = path {
                            commands.entity(*entity).insert(FollowingPath {
                                path: path.nodes.clone(), ..Default::default()
                            });
                        }
                    },
                    Some(UnitDirective::OpenGate) => {
                        let ActionSelector::Target(path) = selector else { continue };
                        if let Some(path) = path {
                            commands.entity(*entity).insert(FabricationGate {
                                path: path.nodes.clone(), filter: *flags,
                                limit: if *flags == 1 { 0 }else{ 1 }, ..Default::default()
                            });
                        } else {
                            commands.entity(*entity).remove::<FabricationGate>();
                        }
                    },
                }
                if match mode.as_ref() {
                    ViewMode::Action(source_entity,_,_) => source_entity.eq(entity),
                    _ => false
                } {
                    *mode = std::mem::take(&mut previous_mode);
                    next_state.set(GlobalState::from(mode.as_ref()));
                }
            },
            InteractionEvent::EnterMode(Some(next_mode)) => {
                *previous_mode = std::mem::replace(&mut mode, next_mode.clone());
                next_state.set(GlobalState::from(mode.as_ref()));
            },
            InteractionEvent::EnterMode(None) => {
                *mode = std::mem::take(&mut previous_mode);
                next_state.set(GlobalState::from(mode.as_ref()));
            },
            InteractionEvent::Start(stage) => {
                next_state.set(GlobalState::Running);
            },
            InteractionEvent::Exit => {
                exit.send(bevy::app::AppExit);
            }
        }
    }
}