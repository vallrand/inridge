use bevy::prelude::*;
use bevy::utils::{HashSet, FloatOrd};
use crate::common::loader::AssetBundle;
use crate::common::noise::{MurMurHash, WeightTable};
use crate::common::adjacency::breadth_first_search;
use crate::logic::{Agent, GridTileIndex, MapGrid, GroupLink, NetworkGroupList, EconomySummary};
use crate::logic::{UnderConstruction, Suspended, MatterBinding, FabricationGate, UnitDirective};
use crate::scene::{UnitBlueprint, BlueprintAssetBundle, GlobalState};
use crate::interaction::{InteractionEvent, ActionSelector, ViewMode, path::ActionPath};
use crate::interface::construct::validate_construction;

pub fn evaluate_end_condition(
    query: Query<&Agent>,
    mut mode: ResMut<ViewMode>,
    mut next_state: ResMut<NextState<GlobalState>>,
){
    let mut mask: u8 = 0;
    for agent in query.iter() {
        mask |= match agent {
            Agent::Player => 0x1,
            &Agent::AI(agent_index) => 1 << agent_index
        };
    }
    if mask == 0x1 || mask & 0x1 == 0 {
        next_state.set(GlobalState::Menu);
        *mode = ViewMode::Menu;
    }
}

pub struct HeuristicContext {
    any_construction: bool,
    any_gate: bool,
}

#[derive(Clone)]
pub enum Heuristic {
    Disabled,
    Neutral,
    Economy(i32),
    Military(i32),
    Civilian(i32)
}
impl Heuristic {
    pub fn from_toggle(blueprint: &UnitBlueprint, _settings: &StrategySettings, summary: &EconomySummary, toggle: bool) -> Self {
        let delta = match blueprint.matter.as_ref() {
            Some(MatterBinding::Consumption(consumption)) => -consumption.quota,
            Some(MatterBinding::Production(production)) => production.efficiency,
            _ => 0
        };
        let prev_delta = summary.matter_production - summary.matter_consumption;
        let next_delta = prev_delta + if toggle { delta }else{ -delta };

        if prev_delta <= 0 && next_delta > prev_delta {
            Heuristic::Economy(next_delta - prev_delta)
        } else if toggle && next_delta > 0 {
            Heuristic::Neutral
        } else {
            Heuristic::Disabled
        }
    }
    pub fn from_construct(blueprint: &UnitBlueprint, settings: &StrategySettings, summary: &EconomySummary, context: &HeuristicContext) -> Self {
        let delta = match blueprint.matter.as_ref() {
            Some(MatterBinding::Consumption(consumption)) => -consumption.quota,
            Some(MatterBinding::Production(production)) => production.efficiency,
            Some(MatterBinding::Collection(storage)) => 0 * storage.discharge,
            None => 0
        };
        let prev_delta = summary.matter_production - summary.matter_consumption;
        let prev_storage = summary.matter_reservation + prev_delta;
        let next_delta = prev_delta + delta;
        if context.any_construction || prev_delta <= 0 && prev_storage <= 0 || next_delta < 0 {
            Heuristic::Disabled
        } else {
            let military = 2 * (blueprint.military.is_some() as i32) +
            blueprint.unit.as_ref().map_or(0, |fabrication|if fabrication.group == 1 { 1 }else{ 0 });
            let upgrade = blueprint.upgrade.is_some() as i32;

            if prev_delta < settings.low_matter_threshold && next_delta > prev_delta {
                Heuristic::Economy(next_delta - prev_delta)
            } else if prev_delta >= settings.low_matter_threshold && military > 0 {
                Heuristic::Military(military)
            } else if prev_delta >= settings.low_matter_threshold && upgrade > 0 {
                Heuristic::Civilian(upgrade)
            } else {
                Heuristic::Neutral
            }
        }
    }
    pub fn from_action(_blueprint: &UnitBlueprint, settings: &StrategySettings, summary: &EconomySummary, context: &HeuristicContext) -> Self {
        let matter_delta = summary.matter_production - summary.matter_consumption;
        if context.any_gate {
            Heuristic::Disabled
        } else if matter_delta > settings.low_matter_threshold {
            Heuristic::Military(1)
        } else {
            Heuristic::Neutral
        }
    }
    pub fn weight(&self) -> i32 { match self {
        Heuristic::Disabled => -1,
        Heuristic::Neutral => 0,
        Heuristic::Economy(score) | Heuristic::Military(score) | Heuristic::Civilian(score) => *score
    } }
}

#[derive(Resource, serde::Deserialize, Clone, Default)]
pub struct StrategySettings {
    pub low_matter_threshold: i32,
}

pub fn strategical_planning_phase(
    time: Res<Time>,
    settings: Res<StrategySettings>,
    mut rng: Local<MurMurHash>,
    mut events: EventWriter<InteractionEvent>,
    blueprints: Res<Assets<UnitBlueprint>>,
    blueprint_bundle: Res<AssetBundle<BlueprintAssetBundle>>,
    query_grid: Query<(Entity, &MapGrid, &NetworkGroupList)>,
    query_unit: Query<(
        &Handle<UnitBlueprint>, Option<&UnderConstruction>, Option<&Suspended>, Option<&FabricationGate>
    )>,
    query_target: Query<(Entity, &Agent, &GridTileIndex), With<GroupLink>>,
){
    for (parent, grid, groups) in query_grid.iter() {
        for group in groups.iter() {
            let Agent::AI(_agent_index) = group.agent else { continue };
            let context = HeuristicContext {
                any_construction: group.list.iter()
                .any(|item|query_unit.get_component::<UnderConstruction>(item.1).is_ok()),
                any_gate: group.list.iter()
                .any(|item|query_unit.get_component::<FabricationGate>(item.1).is_ok()),
            };
            
            let mut candidates: Vec<(Heuristic, InteractionEvent)> = Vec::new();
            let mut visited: HashSet<usize> = Default::default();
            for &(index, entity) in group.list.iter() {
                let Ok((
                    handle, construction, suspended, gate,
                )) = query_unit.get(entity) else { continue };
                let Some(blueprint) = blueprints.get(handle) else { continue };
                if construction.is_some() { continue; }

                candidates.push((
                    Heuristic::from_toggle(blueprint, &settings, &group.summary, suspended.is_some()),
                    InteractionEvent::Toggle(entity)
                ));

                for handle in blueprint_bundle.unit_blueprints.iter() {
                    let Some(next_blueprint) = blueprints.get(handle) else { continue };
                    if !validate_construction(
                        next_blueprint, Some(blueprint), &group.agent, grid, groups, index
                    ) { continue; }

                    candidates.push((
                        Heuristic::from_construct(next_blueprint, &settings, &group.summary, &context),
                        InteractionEvent::Construct(group.agent, parent, index, handle.clone())
                    ));
                }

                if let Some(UnitDirective::OpenGate) = blueprint.action {
                    let mut min_distance_squared = f32::MAX;
                    let mut min_index = index;
                    for (_entity, agent, tile_index) in query_target.iter() {
                        if group.agent.eq(agent) { continue; }
                        let distance_squared = grid.tiles[tile_index.0].transform.translation
                        .distance_squared(grid.tiles[index].transform.translation);
                        if distance_squared < min_distance_squared {
                            min_distance_squared = distance_squared;
                            min_index = tile_index.0;
                        }
                    }
                    if min_index != index {
                        let target_position = grid.tiles[min_index].transform.translation;
                        let nodes = breadth_first_search(
                            &index,
                            |&index|grid.graph.neighbors(index).unwrap().iter()
                            .filter(|&i|
                                grid.tiles[*i].is_empty() || grid.tiles[*i].reference.is_some()
                            ),
                            |index|
                            Some(FloatOrd(grid.tiles[*index].transform.translation.distance_squared(target_position)))
                        );
                        candidates.push((
                            Heuristic::from_action(blueprint, &settings, &group.summary, &context),
                            InteractionEvent::Execute(entity,  ActionSelector::Target(Some(ActionPath{ nodes })), 0x1)
                        ));
                    }
                }

                for &adjacent in grid.graph.neighbors(index).unwrap().iter() {
                    if grid.tiles[adjacent].is_empty() {
                        visited.insert(adjacent);
                    }
                }
            }
            for index in visited.into_iter() {
                for handle in blueprint_bundle.unit_blueprints.iter() {
                    let Some(next_blueprint) = blueprints.get(handle) else { continue };
                    if !validate_construction(
                        next_blueprint, None, &group.agent, grid, groups, index
                    ) { continue; }

                    candidates.push((
                        Heuristic::from_construct(next_blueprint, &settings, &group.summary, &context),
                        InteractionEvent::Construct(group.agent, parent, index, handle.clone())
                    ));
                }
            }
            let mut weight_table: WeightTable<InteractionEvent, u32> = Default::default();
            for (heuristic, event) in candidates.into_iter() {
                let weight = heuristic.weight();
                if weight >= 0 {
                    weight_table.add(event, weight as u32 + 1);
                }
            }

            rng.next(time.elapsed().as_millis() as u64);
            let random_index = rng.next_u32(weight_table.total());
            let Some(event) = weight_table.take(random_index) else { continue };
            events.send(event);
        }
    }
}