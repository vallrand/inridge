use bevy::prelude::*;
use crate::common::adjacency::breadth_first_search;
use crate::logic::MapGrid;
use super::{GridSelection, ViewMode, ActionSelector};

#[derive(Deref, DerefMut, PartialEq, Clone, Default)]
pub struct ActionPath {
    pub nodes: Vec<usize>,
}
impl ActionPath {
    pub fn simplify(&mut self, grid: &MapGrid){
        let mut i: usize = 0;
        while i < self.nodes.len() {
            i += 1;
            for j in (i..self.nodes.len()).rev() {
                if grid.graph.contains(self.nodes[i-1], self.nodes[j]) {
                    self.nodes.drain(i..j);
                    break;
                }
            }
        }
    }
}

pub fn select_action_path(
    mut mode: ResMut<ViewMode>,
    query_grid: Query<(&MapGrid, Ref<GridSelection>)>
){
    let ViewMode::Action(_target_index, origin_index, selector) = mode.bypass_change_detection() else { return };
    let origin_index = *origin_index;
    let Ok((grid, selection)) = query_grid.get_single() else { return };
    if !selection.is_changed() { return; }

    let (selected_path, passthrough) = match selector {
        ActionSelector::FollowPath(path) => (path, false),
        ActionSelector::Target(path) => (path, true)
    };

    let prev_index = selected_path.as_ref().and_then(|path|path.nodes.last()).unwrap_or(&origin_index);
    let nodes = breadth_first_search(
        prev_index,
        |&index|grid.graph.neighbors(index).unwrap().iter()
        .filter(|&i|origin_index.eq(i) || match passthrough {
            false => grid.tiles[*i].is_empty(),
            true => grid.tiles[*i].is_empty() || grid.tiles[*i].reference.is_some()
        }),
        |index| if selection.0.eq(index) { None } else {
        Some(bevy::utils::FloatOrd(grid.tiles[*index].transform.translation.distance_squared(grid.tiles[selection.0].transform.translation)))
    });

    if let Some(path) = selected_path.as_mut() {
        let (tail, head) = nodes.iter()
        .enumerate().rev()
        .find_map(|(i,lhs)|
            path.nodes.iter().position(|rhs|rhs==lhs).map(|j|(i,j))
        ).unwrap_or((0, path.nodes.len() - 1));

        path.nodes.splice(head.., nodes[tail..].into_iter().cloned());
        path.simplify(&grid);
        if path.len() < 2 { selected_path.take(); }
    } else {
        let mut path = ActionPath{ nodes };
        path.simplify(&grid);
        if path.len() >= 2 { selected_path.replace(path); }
    }
    mode.set_changed();
}