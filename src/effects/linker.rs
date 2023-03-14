use std::collections::BTreeMap;
use bevy::prelude::*;
use crate::common::loader::AssetBundle;
use crate::scene::ModelAssetBundle;
use crate::effects::animation::AnimationSettings;
use crate::logic::{MapGrid,ConstructionEvent};

#[derive(Component, Default, Deref, DerefMut)]
pub struct TileConnectors(BTreeMap<usize, Entity>);

pub struct ConnectorAnimation {
    pub entity: Entity,
    pub scale: f32,
    pub timer: Timer,
    pub reverse: bool,
}

pub fn link_structures(
    settings: Res<AnimationSettings>,
    time: Res<Time>,
    mut cache: Local<Option<Handle<Mesh>>>,
    mut transitions: Local<Vec<ConnectorAnimation>>,
    scenes: Res<Assets<Scene>>,
    mut events: EventReader<ConstructionEvent>,
    mut commands: Commands,
    model_bundle: Res<AssetBundle<ModelAssetBundle>>,
    mut query_grid: Query<(&MapGrid, &mut TileConnectors)>,
    mut query: Query<&mut Transform>,
){
    if let None = *cache {
        let scene = model_bundle.extracted.get("bridge".into())
        .and_then(|(handle,_)|scenes.get(handle)).unwrap();
        for entity in scene.world.iter_entities() {
            let Some(handle) = scene.world.get::<Handle<Mesh>>(entity.id()) else { continue };
            cache.replace(handle.clone());
            break;
        }
    }

    transitions.retain_mut(|connector|{
        connector.timer.tick(time.delta());
        let fraction = if connector.reverse { 1.0 - connector.timer.percent() }else{ connector.timer.percent() };
        if let Ok(mut transform) = query.get_mut(connector.entity) {
            transform.scale = Vec3::new(1.0, fraction, fraction) * connector.scale;
        }
        if connector.timer.just_finished() && connector.reverse {
            commands.entity(connector.entity).despawn();
        }
        !connector.timer.finished()
    });

    for event in events.iter() {
        match event {
            &ConstructionEvent::Begin { parent, index, .. } |
            &ConstructionEvent::Assemble { parent, index, .. } => {
                let Ok((grid, mut connectors)) = query_grid.get_mut(parent) else { continue };
                let origin_tile = &grid.tiles[index];
                let Some(origin_group) = grid.visited.get(&index) else { continue };
                for &neighbor in grid.graph.neighbors(index).unwrap().iter() {
                    let target_tile = &grid.tiles[neighbor];
                    if target_tile.is_empty() || grid.visited.get(&neighbor)
                        .map_or(true,|group|group != origin_group) { continue; }

                    let key = grid.graph.edge_key(index, neighbor).unwrap();
                    if connectors.contains_key(&key) { continue; }

                    let center = 0.5 * (origin_tile.transform.translation + target_tile.transform.translation);
                    let normal = center.normalize();
                    let distance = origin_tile.transform.translation.distance(target_tile.transform.translation);

                    let mut transform = Transform::from_translation(center.lerp(normal, 1.0 - (3.0 as f32).sqrt().recip()))
                    .looking_to(target_tile.transform.translation - origin_tile.transform.translation, normal)
                    .with_scale(Vec3::splat(0.0));
                    transform.rotate_local_axis(Vec3::Y, std::f32::consts::FRAC_PI_2);

                    let entity = commands.spawn(MaterialMeshBundle{
                        material: model_bundle.static_material.clone(),
                        mesh: cache.as_ref().unwrap().clone(),
                        transform, ..Default::default()
                    }).set_parent(parent).id();

                    connectors.insert(key, entity);
                    transitions.push(ConnectorAnimation{
                        entity, scale: distance / 2.0, reverse: false,
                        timer: Timer::from_seconds(settings.bridge_transition_duration, TimerMode::Once)
                    });
                }
            },
            &ConstructionEvent::Dismantle { parent, index, .. } => {
                let Ok((grid, mut connectors)) = query_grid.get_mut(parent) else { continue };
                for &neighbor in grid.graph.neighbors(index).unwrap().iter() {
                    let key = grid.graph.edge_key(index, neighbor).unwrap();
                    let Some(entity) = connectors.remove(&key) else { continue };

                    let transform = query.get(entity).unwrap();
                    let elapsed = if let Some(index) = transitions.iter().position(|transition|transition.entity == entity) {
                        let previous = transitions.remove(index);
                        previous.timer.duration().saturating_sub(previous.timer.elapsed())
                    } else { std::time::Duration::ZERO };
                    let mut timer = Timer::from_seconds(settings.bridge_transition_duration, TimerMode::Once);
                    timer.set_elapsed(elapsed);
                    transitions.push(ConnectorAnimation{
                        entity, scale: transform.scale.x, reverse: true, timer
                    });
                }
            }
        }
    }
}