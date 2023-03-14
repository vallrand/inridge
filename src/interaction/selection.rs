use bevy::prelude::*;
use crate::common::raycast;
use crate::common::spatial::Intersect;
use crate::logic::MapGrid;

#[derive(Component, Deref, DerefMut, Clone, Copy, PartialEq, Eq)]
pub struct GridSelection(pub usize);

pub fn update_grid_selection(
    mut commands: Commands,
    view_query: Query<&raycast::RaycastSource, With<Camera>>,
    query: Query<(Entity, &MapGrid, Option<&GridSelection>, &raycast::HitArea, &GlobalTransform)>,
    query_clear: Query<Entity, With<GridSelection>>,
){
    let source = view_query.single();
    for (entity, grid, selection, hit_area, transform) in &query {
        let local_ray = transform.compute_matrix().inverse() * source.ray;
        let raycast::HitArea::Sphere(sphere) = hit_area else { continue };
        let Some(intersection) = local_ray.intersect(sphere) else { continue };

        let previous = selection.as_ref().map(|selection|selection.0).unwrap_or(0);
        let closest = grid.find_closest(previous, intersection.position.into());
        if previous != closest {
            println!("select [{}]", closest);
            for entity in query_clear.iter() { commands.entity(entity).remove::<GridSelection>(); }
            commands.entity(entity).insert(GridSelection(closest));
        }
        if let Some(unit) = grid.tiles[closest].reference {
            if !query_clear.contains(unit) || previous != closest {
                commands.entity(unit).insert(GridSelection(closest));
            }
        }
        break;
    }
}
