use bevy::prelude::*;
use crate::common::loader::AssetBundle;
use crate::effects::animation::AnimationSettings;
use crate::materials::{ColorUniform, ScanlineEffectMaterial, ModelEffectLayeredMaterial};
use crate::logic::{GroupLink, MapGrid, GridTileIndex, MatterBinding, UnderConstruction, Suspended};
use crate::scene::EffectAssetBundle;
use crate::effects::outline::BorderOutline;

#[derive(Component, Clone)]
pub struct DeficitEffectAnimation {
    entity: Entity,
    intensity: f32,
}

pub fn animate_unit_condition_deficit(
    time: Res<Time>,
    settings: Res<AnimationSettings>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    effect_bundle: Res<AssetBundle<EffectAssetBundle>>,
    query_grid: Query<&MapGrid>,
    mut query_unit: Query<(
        Entity, &Parent, &GridTileIndex, Option<&mut DeficitEffectAnimation>,
        Option<&MatterBinding>, Option<&UnderConstruction>, Option<&Suspended>, Option<&GroupLink>
    )>,
    children: Query<&Children>,
    mut query: Query<&mut ColorUniform, Or<(
        With<Handle<ModelEffectLayeredMaterial>>,
        With<Handle<ScanlineEffectMaterial>>
    )>>,
){
    for (
        entity, parent, tile_index, effect,
        matter, construction, suspended, group
    ) in query_unit.iter_mut() {
        let deficit = match (group, suspended, construction, matter) {
            (Some(_), None, Some(construction), _) => !construction.active(),
            (Some(_), None, None, Some(MatterBinding::Consumption(consumption))) => !consumption.active(),
            _ => false,
        };
        match (deficit, effect) {
            (true, None) => {
                let Ok(grid) = query_grid.get(parent.get()) else { continue };
                let effect = commands.spawn(MaterialMeshBundle{
                    mesh: meshes.add(BorderOutline::from_single(grid, tile_index.0)
                        .with_stroke(grid.tiles[tile_index.0].transform.scale.y, 0.0, true).into()),
                    material: effect_bundle.material_deficit.clone(), ..Default::default()
                }).insert((
                    bevy::pbr::NotShadowCaster,
                    bevy::pbr::NotShadowReceiver,
                ))
                .insert(ColorUniform::from(Color::NONE))
                .set_parent(parent.get()).id();
                commands.entity(entity).insert(DeficitEffectAnimation {
                    entity: effect, intensity: 0.0,
                });
            },
            (false, None) => {},
            (deficit, Some(mut effect)) => {
                let delta: f32 = time.delta_seconds() / settings.condition_transition_duration;
                effect.intensity = (effect.intensity + if deficit { delta }else{ -delta }).clamp(0.0, 1.0);

                for entity in children.iter_descendants(entity) {
                    let Ok(mut uniform) = query.get_mut(entity) else { continue };
                    uniform.color.set_b(effect.intensity);
                }

                if effect.intensity == 0.0 && !deficit {
                    commands.entity(effect.entity).despawn();
                    commands.entity(entity).remove::<DeficitEffectAnimation>();
                } else {
                    let Ok(mut uniform) = query.get_mut(effect.entity) else { continue };
                    uniform.color.set_a(effect.intensity);
                }
            }
        }
    }
}