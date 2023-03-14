use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy::asset::Asset;
use bevy::scene::{InstanceInfo, SceneSpawnError, SceneInstance};
use bevy::ecs::entity::EntityMap;
use bevy::ecs::reflect::ReflectMapEntities;

///https://docs.rs/bevy/latest/bevy/scene/struct.Scene.html#method.write_to_world_with
pub fn clone_entities(source_world: &World, world: &mut World, filter: impl Fn(Entity) -> bool) -> Result<InstanceInfo, SceneSpawnError> {
    let type_registry = &world.resource::<AppTypeRegistry>().clone();
    let mut instance_info = InstanceInfo { entity_map: EntityMap::default() };
    let type_registry = type_registry.read();
    for archetype in source_world.archetypes().iter() {
        for scene_entity in archetype.entities() {
            if !filter(scene_entity.entity()) { continue; }
            let entity = *instance_info
                .entity_map
                .entry(scene_entity.entity())
                .or_insert_with(|| world.spawn_empty().id());
            for component_id in archetype.components() {
                let component_info = source_world
                    .components()
                    .get_info(component_id)
                    .expect("component_ids in archetypes should have ComponentInfo");

                let reflect_component = type_registry
                    .get(component_info.type_id().unwrap())
                    .ok_or_else(|| SceneSpawnError::UnregisteredType {
                        type_name: component_info.name().to_string(),
                    })
                    .and_then(|registration| {
                        registration.data::<ReflectComponent>().ok_or_else(|| {
                            SceneSpawnError::UnregisteredComponent {
                                type_name: component_info.name().to_string(),
                            }
                        })
                    })?;
                reflect_component.copy(&source_world, world, scene_entity.entity(), entity);
            }
        }
    }
    for registration in type_registry.iter() {
        if let Some(map_entities_reflect) = registration.data::<ReflectMapEntities>() {
            map_entities_reflect
                .map_entities(world, &instance_info.entity_map)
                .unwrap();
        }
    }
    Ok(instance_info)
}

#[derive(Component, Clone, Default, Deref, DerefMut)]
pub struct EntityLookupTable(HashMap<String, Entity>);
pub fn cache_scene_entity_lookup_table(
    mut commands: Commands,
    query: Query<&Parent, Changed<SceneInstance>>,
    children: Query<&Children>,
    query_keys: Query<&Name>
){
    for parent in query.iter() {
        let mut lookup_table: HashMap<String, Entity> = HashMap::new();
        for entity in children.iter_descendants(parent.get()) {
            let Ok(name) = query_keys.get(entity) else { continue };
            lookup_table.insert(name.into(), entity);
        }
        commands.entity(parent.get()).insert(EntityLookupTable(lookup_table));
    }
}

pub fn replace_components<T: Asset>(scene: &mut Scene, marker: &Handle<T>, bundle: impl Bundle + Clone){
    let entities: Vec<Entity> = scene.world.query::<(Entity, &Handle<T>)>().iter(&scene.world)
    .filter_map(|(entity, handle)|
        if handle.id() == marker.id() { Some(entity) }else { None }
    ).collect();
    for entity in entities.into_iter() {
        scene.world.entity_mut(entity).remove::<Handle<T>>()
        .insert(bundle.clone());
    }
}