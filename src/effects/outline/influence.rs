use bevy::prelude::*;
use crate::common::geometry::merge_meshes;
use crate::common::animation::{Animator,Track,AnimationStateMachine,StateMachineTransition};
use crate::common::loader::AssetBundle;
use crate::materials::{ColorUniform, UnlitMaterial};
use crate::logic::{MapGrid, UpgradeDistribution};
use crate::scene::bundles::effects::EffectAssetBundle;
use crate::interaction::{GridSelection, SelectionState};
use super::border::BorderOutline;

pub fn update_grid_affected_highlight(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<UnlitMaterial>>,
    effect_bundle: Res<AssetBundle<EffectAssetBundle>>,
    query_grid: Query<&MapGrid>,
    query_unit: Query<(&Parent, &UpgradeDistribution), With<GridSelection>>,
    mut previous_selection: Local<Option<(Entity, u64)>>,
){
    let selected = query_unit.get_single().ok();

    let hash = selected.and_then(|(_, UpgradeDistribution { list, .. })|list.reflect_hash()).unwrap_or_default() as u64;

    if previous_selection.map(|(_,prev_hash)|prev_hash == hash).unwrap_or(false) { return; }

    if let Some((entity,_)) = previous_selection.take() {
        if let Some(mut commands) = commands.get_entity(entity) {
            commands.insert(SelectionState::None);
        }
    }

    let Some((parent, UpgradeDistribution { list, .. })) = selected else { return };
    if list.is_empty() { return; }
    let Ok(grid) = query_grid.get(parent.get()) else { return };
    let height = grid.tiles[list[0]].transform.scale.y;
    let border = commands.spawn((
        SpatialBundle::default(),
        meshes.add(merge_meshes(&[
            BorderOutline::from_group(&grid, &list, false)
            .with_uv(0.5, 0.0)
            .with_stroke(height * 0.4, 0.0, false).with_offset(0.02).into(),
            BorderOutline::from_group(&grid, &list, false)
            .with_uv(0.5, 1.0)
            .with_stroke(height * 1.2, 0.0, true).into(),
        ], None)),
        materials.add(UnlitMaterial{
            color: Color::rgb(0.8,1.0,0.4),
            diffuse: Some(effect_bundle.glow.clone()),
            double_sided: true,
            ..Default::default()
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
    commands.entity(parent.get()).add_child(border);
    previous_selection.replace((border, hash));
}