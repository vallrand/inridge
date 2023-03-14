use bevy::prelude::*;
use crate::common::geometry::merge_meshes;
use crate::effects::outline::BorderOutline;
use crate::logic::{UnderConstruction,MapGrid,GridTileIndex};
use crate::materials::ReconstructEffectMaterial;

#[derive(Component, Clone)]
pub enum StructureCondition {
    Construction {
        intensity: f32,
        effect: Entity,
        material: Handle<ReconstructEffectMaterial>,
    }
}

pub fn animate_reconstruction(
    time: Res<Time>,
    mut commands: Commands,
    mut materials: ResMut<Assets<ReconstructEffectMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    query_grid: Query<&MapGrid>,
    mut query_unit: Query<(Entity, &Parent, &GridTileIndex, Option<&UnderConstruction>, Option<&mut StructureCondition>)>,
){
    let intro_duration = 2.0;
    let outro_duration = 1.0;

    for (entity, parent, tile_index, construction, mut condition) in query_unit.iter_mut() {
        let Ok(grid) = query_grid.get(parent.get()) else { continue };
        let tile = &grid.tiles[tile_index.0];

        match (construction, condition.as_deref_mut()) {
            (None, Some(StructureCondition::Construction { effect, material, intensity })) => {
                *intensity = (*intensity - time.delta_seconds() / outro_duration).min(1.0);
                materials.get_mut(material).unwrap().threshold = *intensity;

                if *intensity < 0.0 {
                    commands.entity(*effect).remove_parent();
                    commands.entity(*effect).despawn();
                    commands.entity(entity).remove::<StructureCondition>();
                }
            },
            (Some(_), Some(StructureCondition::Construction { material, intensity, .. })) => {
                *intensity = (*intensity + time.delta_seconds() / intro_duration).min(1.0);
                materials.get_mut(material).unwrap().threshold = *intensity;
            },
            (Some(_), None) => {
                let material = materials.add(ReconstructEffectMaterial{
                    color: Color::rgb(0.8,1.0,0.6), glow_color: Color::rgb(1.0,1.0,0.8),
                    domain: Vec2::new(16.0, 16.0), alpha_mode: AlphaMode::Add, vertical_fade: Vec2::new(0.1, 0.4),
                    threshold: 0.0,
                });
                let effect_entity = commands.spawn(MaterialMeshBundle{
                    mesh: meshes.add(merge_meshes(&[
                        BorderOutline::from_single(grid, tile_index.0)
                                .with_stroke(tile.transform.scale.y * 2.0, 0.0, true).into(),
                        BorderOutline::from_single(grid, tile_index.0)
                                .with_stroke(tile.transform.scale.y * 0.5, 0.0, false)
                                .with_uv(0.0, -1.0).with_offset(tile.transform.scale.y * 0.1).into(),
                    ], None)),
                    material: material.clone(),
                    ..Default::default()
                }).insert((
                    bevy::pbr::NotShadowCaster,
                    bevy::pbr::NotShadowReceiver,
                )).set_parent(parent.get()).id();

                commands.entity(entity).insert(StructureCondition::Construction{ effect: effect_entity, material, intensity: 0.0 });
            },
            (_, _) => {}
        }
    }
}