use bevy::prelude::*;
use bevy::math::Vec3A;
use bevy::utils::{HashMap,hashbrown::hash_map::Entry};
use super::spatial::{ray::Ray, aabb::AABB, bvh::BVHTree, Intersect, intersect::RaycastHit, sphere::bounding_sphere_gaertner};

use super::geometry::{plane::Plane,sphere::Sphere};
#[derive(Component)]
pub enum HitArea {
    Plane(Plane),
    Sphere(Sphere)
}

#[derive(Component, Default)]
pub struct RaycastSource {
    pub ray: Ray,
    pub hits: Vec<(Entity, RaycastHit)>,
    pub group: u32,
}

fn update_cursor_position(
    mut cursor_move_events: EventReader<CursorMoved>,
    primary_window: Query<Entity, With<bevy::window::PrimaryWindow>>,
    touches: Res<Touches>,
    mut query: Query<(&mut bevy::ui::RelativeCursorPosition, &Camera)>
){
    let primary_window = primary_window.get_single().ok();
    for (mut cursor, camera) in &mut query {
        let bevy::render::camera::RenderTarget::Window(window) = camera.target else { continue };
        let Some(window) = window.normalize(primary_window) else { continue };

        let Some(absolute_position) = (match cursor_move_events.iter().last() {
            Some(cursor_moved) => if cursor_moved.window == window.entity() {
                Some(cursor_moved.position)
            } else {
                None
            },
            None => touches.iter().last().map(|touch| touch.position())
        }) else { continue };
        //TODO: move ndc calculation here?
        cursor.normalized = Some(absolute_position);
    }
}

fn recalculate_rays(
mut query: Query<(
    &mut RaycastSource,
    &GlobalTransform,
    Option<&Camera>,
    Option<&bevy::ui::RelativeCursorPosition>
)>){
    for (mut source, transform, camera, cursor) in &mut query {
        source.ray = if let Some(camera) = camera {
            let cursor = cursor.and_then(|cursor| cursor.normalized);
            Ray::from_screenspace(transform.compute_matrix(), camera, cursor).unwrap_or_default()
        } else {
            Ray::from_transform(transform.compute_matrix())
        }
    }
}

#[derive(Component, Default)]
pub struct RaycastTarget {
    pub aabb: AABB
}

#[derive(Resource, Default)]
pub struct BoundingHierarchy {
    indices: HashMap<Entity, usize>,
    tree: BVHTree<Entity>
}

fn recalculate_bounding_hierarchy(
    meshes: Res<Assets<Mesh>>,
    mut bounding_volumes: Local<HashMap<bevy::asset::HandleId, Sphere>>,
    mut events: EventReader<AssetEvent<Mesh>>,

    mut bounding_hierarchy: ResMut<BoundingHierarchy>,
    mut removals: RemovedComponents<RaycastTarget>,
    mut query: Query<(
        Entity,
        &mut RaycastTarget,
        &ComputedVisibility,
        &GlobalTransform,
        &Handle<Mesh>,
        Option<&HitArea>,
    ), Or<(Changed<ComputedVisibility>, Changed<GlobalTransform>)>>
){
    fn recalculate_bounding_volume(handle: &Handle<Mesh>, meshes: &Assets<Mesh>) -> Option<Sphere> {
        use bevy::render::mesh::VertexAttributeValues;
        let Some(mesh) = meshes.get(handle) else { return None };
        let Some(VertexAttributeValues::Float32x3(vertices)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) else { return None };
        let (center, radius) = bounding_sphere_gaertner(&vertices, f32::EPSILON);
        Some(Sphere{ origin: Vec3A::from(center), radius })
    }
    for event in events.iter() {
        match event {
            AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                bounding_volumes.entry(handle.id())
                .and_replace_entry_with(|_,_|recalculate_bounding_volume(handle, &meshes));
            },
            AssetEvent::Removed { handle } => {
                bounding_volumes.remove(&handle.id());
            }
        }
    }

    let BoundingHierarchy { indices, tree } = bounding_hierarchy.as_mut();
    for (
        entity, mut target, visibility,
        transform, mesh, bounds
    ) in &mut query {
        if !visibility.is_visible_in_hierarchy() { continue; }
        if let Some(bounds) = bounds {
            target.aabb = match bounds {
                HitArea::Sphere(bounding_sphere) => AABB::from_bounding_sphere(bounding_sphere, &transform.affine()),
                _ => continue
            };
        } else {
            let Some(bounding_sphere) = (match bounding_volumes.entry(mesh.id()) {
                Entry::Occupied(entry) => Some(entry.into_mut()),
                Entry::Vacant(entry) => 
                    recalculate_bounding_volume(mesh, &meshes).map(|sphere|entry.insert(sphere))
            }) else { continue };
            target.aabb = AABB::from_bounding_sphere(bounding_sphere, &transform.affine());
        }
        indices.entry(entity).and_modify(|node_index|{
            *node_index = tree.update(*node_index, target.aabb, 0.0);
        }).or_insert_with(||{
            tree.insert(entity, target.aabb, 0.0)
        });
    }
    for entity in removals.iter() {
        let Some(node_index) = indices.remove(&entity) else { continue };
        tree.remove(node_index);
    }
}

fn recalculate_raycasts(
    bounding_hierarchy: Res<BoundingHierarchy>,
    meshes: Res<Assets<Mesh>>,
    mut source_query: Query<&mut RaycastSource>,
    target_query: Query<(
        &GlobalTransform,
        &Handle<Mesh>,
        Option<&HitArea>,
    ), With<RaycastTarget>>,
){
    for mut source in &mut source_query {
        let mut list = std::collections::BTreeMap::new();
        for &entity in bounding_hierarchy.tree.query(&source.ray) {
            let Ok((transform, mesh, bounds)) = target_query.get(entity) else { continue };
            let transform = transform.compute_matrix();
            let local_ray = transform.inverse() * source.ray;
            let Some(mut intersection) = (
                if let Some(bounds) = bounds {
                    match bounds {
                        HitArea::Sphere(sphere) => local_ray.intersect(sphere),
                        _ => None
                    }
                } else {
                    meshes.get(mesh).and_then(|mesh| local_ray.intersect(mesh))
                }
            ) else { continue };
            intersection *= transform;
            list.insert(bevy::utils::FloatOrd(intersection.distance), (entity, intersection));
        }
        source.hits = list.into_values().collect();
    }
}

pub struct RaycastPlugin; impl Plugin for RaycastPlugin {
    fn build(&self, app: &mut App) {
        use bevy::render::view::VisibilitySystems;
        app
        .init_resource::<BoundingHierarchy>()
        .add_systems((
            update_cursor_position
                .before(recalculate_rays),
            recalculate_rays,
            recalculate_bounding_hierarchy
                .after(VisibilitySystems::VisibilityPropagate),
            recalculate_raycasts
                .after(recalculate_bounding_hierarchy)
                .after(recalculate_rays)
        ).in_base_set(CoreSet::PostUpdate));
    }
}