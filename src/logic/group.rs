use bevy::prelude::*;
use super::agent::Agent;
use super::economy::EconomySummary;
use super::terrain::grid::{MapGrid,GridTileIndex};

#[derive(Clone, Default)]
pub struct NetworkGroup {
    pub agent: Agent,
    pub list: Vec<(usize, Entity)>,
    pub summary: EconomySummary,
}

#[derive(Component, Deref, DerefMut, Clone, Default)]
pub struct NetworkGroupList(pub Vec<NetworkGroup>);

#[derive(Component, Deref, DerefMut, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct PriorityOrder(pub u64);

#[derive(Component, Deref, DerefMut, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct GroupLink(pub Option<usize>);

pub fn relink_network_group(
    mut query_grid: Query<(&Children, &mut MapGrid, &mut NetworkGroupList), Changed<MapGrid>>,
    mut query_unit: Query<(Entity, &GridTileIndex, &Agent, &mut GroupLink, &PriorityOrder), With<GroupLink>>,
){
    for (children, mut grid, mut groups) in query_grid.iter_mut() {
        let grid = grid.bypass_change_detection();
        grid.visited.clear();
        groups.clear();

        for &entity in children {
            let Ok((_, &tile_index, &agent, _, _)) = query_unit.get(entity) else { continue };
            if grid.visited.contains_key(&tile_index) { continue; }

            let group_index = groups.len();
            let Some(mut list) = grid.fill_group(*tile_index, group_index, |tile|
                tile.reference.and_then(|entity|query_unit.get(entity).ok())
                .filter(|value|agent.eq(value.2))
            ) else { continue };

            list.sort_by_key(|item|item.4.0);
            let list = list.into_iter()
            .map(|(entity,tile_index,_,_,_)|(**tile_index, entity)).collect();
            let group = NetworkGroup { list, agent, ..Default::default() };
            for (_, entity) in group.list.iter() {
                let Ok(mut group) = query_unit.get_component_mut::<GroupLink>(*entity) else { continue };
                group.replace(group_index);
            }
            groups.push(group);
        }
        println!("relinking network groups ({})", groups.len());
    }
}