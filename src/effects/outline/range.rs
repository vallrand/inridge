use bevy::prelude::*;
use crate::common::loader::AssetBundle;
use crate::common::animation::{Animator, Track, AnimationStateMachine, StateMachineTransition};
use crate::interaction::{GridSelection, SelectionState};
use crate::logic::{MilitaryBinding, MilitarySupply};
use crate::materials::ColorUniform;
use crate::scene::EffectAssetBundle;

pub fn update_military_range(
    mut commands: Commands,
    effect_bundle: Res<AssetBundle<EffectAssetBundle>>,
    query_unit: Query<(Entity, &MilitaryBinding, &MilitarySupply, &GlobalTransform), With<GridSelection>>,
    mut query_transform: Query<&mut Transform>,
    mut previous_selection: Local<Option<(Entity, Entity)>>,
){
    let selected = query_unit.get_single().ok();

    if let (Some(prev), Some((entity, military, supply, _))) = (previous_selection.as_ref(), selected.as_ref()) {
        if prev.0.eq(entity) {
            let radius = military.radius() * supply.range_multipler();
            let Ok(mut transform) = query_transform.get_mut(prev.1) else { return };
            let target_scale = Vec3::splat(2.0 * radius);
            if !transform.scale.abs_diff_eq(target_scale, f32::EPSILON) {
                transform.scale = transform.scale.lerp(target_scale, 0.1);
            }
            return;
        }
    }

    if let Some(prev) = previous_selection.take() {
        commands.entity(prev.1).insert(SelectionState::None);
    }

    let Some((entity, military, supply, transform)) = selected else { return };
    if let MilitaryBinding::Area { .. } = military { return; }
    let radius = military.radius() * supply.range_multipler();

    let effect = commands.spawn((
        SpatialBundle::from_transform(Transform::default()
            .with_translation(transform.translation())
            .with_scale(Vec3::splat(2.0 * radius))
        ),
        effect_bundle.mesh_sphere.clone(),
        effect_bundle.material_fresnel.clone(),
        ColorUniform::from(Color::NONE),
        AnimationStateMachine::new(vec![
            StateMachineTransition::new(SelectionState::None, SelectionState::Hover, 0.2),
            StateMachineTransition::new(SelectionState::Hover, SelectionState::None, 0.2),
        ], SelectionState::None),
        SelectionState::Hover,
        Animator::<ColorUniform>::new()
            .add(Track::from_static(Color::NONE.into()).with_state(SelectionState::None))
            .add(Track::from_static(Color::rgb(0.0,0.8,1.0).into()).with_state(SelectionState::Hover)),
        bevy::pbr::NotShadowCaster,
        bevy::pbr::NotShadowReceiver,
    )).id();
    previous_selection.replace((entity, effect));
}