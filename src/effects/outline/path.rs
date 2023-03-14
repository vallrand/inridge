use bevy::prelude::*;
use crate::common::loader::AssetBundle;
use crate::logic::MapGrid;
use crate::scene::InterfaceAssetBundle;
use crate::materials::{UnlitMaterial, ColorUniform};
use crate::interaction::{ViewMode, ActionSelector, GridSelection};
use super::BorderOutline;

pub fn animate_selected_path(
    mode: Res<ViewMode>,
    mut commands: Commands,
    mut prev: Local<Option<Entity>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<UnlitMaterial>>,
    interface_bundle: Res<AssetBundle<InterfaceAssetBundle>>,
    query_grid: Query<(Entity, &MapGrid), With<GridSelection>>
){
    if !mode.is_changed() { return; }
    if let Some(entity) = prev.take() { commands.entity(entity).despawn(); }

    let (
        ViewMode::Action(_,_, ActionSelector::FollowPath(Some(path))) |
        ViewMode::Action(_,_, ActionSelector::Target(Some(path)))
    ) = mode.as_ref() else { return };

    let Ok((parent, grid)) = query_grid.get_single() else { return };

    
    let height = path.nodes.first().map(|&i|grid.tiles[i].transform.scale.y).unwrap_or_default();
    let entity = commands.spawn((
        SpatialBundle::default(),
        meshes.add(BorderOutline::from_directions(&grid, &path.nodes)
            .with_offset(height * 0.1)
            .with_stroke(height * 1.6, 0.5, false)
            .into()),
        materials.add(UnlitMaterial {
            color: interface_bundle.color_active,
            diffuse: Some(interface_bundle.arrow.clone()),
            ..Default::default()
        }),
        ColorUniform::from(Color::WHITE),
    )).set_parent(parent).id();
    prev.replace(entity);
}