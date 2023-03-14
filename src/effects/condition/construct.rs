use bevy::prelude::*;
use bevy::render::primitives::Aabb;
use crate::common::loader::AssetBundle;
use crate::common::animation::ease::BezierCurve;
use crate::scene::ModelAssetBundle;
use crate::materials::ModelEffectLayeredMaterial;
use crate::logic::{Agent, UnderConstruction};
use super::membership::MembershipSettings;

#[derive(Component)]
pub struct ConstructionEffect {
    pub prev_model_handle: Handle<ModelEffectLayeredMaterial>,
    pub prev_glass_handle: Handle<StandardMaterial>,
    pub model_handle: Handle<ModelEffectLayeredMaterial>,
    pub glass_handle: Handle<StandardMaterial>,
    pub glass_alpha: f32,
}

pub struct ConstructionEffectOptions {
    pub ease: BezierCurve,
    pub dissolve_scale: Vec3,
}
impl Default for ConstructionEffectOptions { fn default() -> Self {Self {
    ease: BezierCurve::new(0.0,0.5,1.0,0.5),
    dissolve_scale: Vec3::new(4.0, 36.0, 4.0),
} } }


pub fn animate_construction(
    membership_settings: Res<MembershipSettings>,
    options: Local<ConstructionEffectOptions>,
    fixed_time: Res<FixedTime>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut effect_materials: ResMut<Assets<ModelEffectLayeredMaterial>>,
    model_asset_bundle: Res<AssetBundle<ModelAssetBundle>>,
    mut set: ParamSet<(
        Query<(Entity, &Children, &Agent, &UnderConstruction, &GlobalTransform), Without<ConstructionEffect>>,
        Query<(&UnderConstruction, &ConstructionEffect)>,
        Query<(Entity, &Children, &ConstructionEffect), Without<UnderConstruction>>,
    )>,
    query_scene: Query<(Entity, &Aabb), With<bevy::scene::SceneInstance>>,
    query_descendants: Query<&Children>,
    query_mesh: Query<
        (Option<&Handle<ModelEffectLayeredMaterial>>, Option<&Handle<StandardMaterial>>),
        Or<(With<Handle<ModelEffectLayeredMaterial>>, With<Handle<StandardMaterial>>)>
    >,
){
    let fraction = fixed_time.accumulated().as_secs_f32() / fixed_time.period.as_secs_f32();

    for (entity, children, agent, construction, transform) in set.p0().iter() {
        let Some((model, aabb)) = children.first().and_then(|&entity| query_scene.get(entity).ok()) else { continue };
 
        let material_handle = membership_settings.get(agent).map(|handles|&handles.0).unwrap_or(&model_asset_bundle.material);
        let prev_model_material = effect_materials.get(material_handle).unwrap();
        let prev_glass_material = materials.get(&model_asset_bundle.glass_material).unwrap();

        let normal = transform.affine().transform_vector3(Vec3::Y / (aabb.max().y - aabb.min().y));     
        let plane = transform.affine().transform_point3(Vec3::Y * aabb.min().y);
        let progress = options.ease.calculate(construction.calculate(fraction));
        let material = ModelEffectLayeredMaterial{
            alpha_threshold: progress * (1.0 + prev_model_material.dissolve_offset.x),
            dissolve: true, damage: false,
            dissolve_plane: normal.extend(plane.dot(normal)),
            noise_domain: options.dissolve_scale,
            ..prev_model_material.clone()
        };
        let glass_material = StandardMaterial {
            base_color: prev_glass_material.base_color.clone().with_a(0.0),
            ..prev_glass_material.clone()
        };
        let effect = ConstructionEffect {
            prev_model_handle: material_handle.clone(),
            prev_glass_handle: model_asset_bundle.glass_material.clone(),
            model_handle: effect_materials.add(material),
            glass_alpha: prev_glass_material.base_color.a(),
            glass_handle: materials.add(glass_material),
        };

        for entity in query_descendants.iter_descendants(model) {
            match query_mesh.get(entity) {
                Ok((Some(handle), None)) => {
                    if handle.id() == effect.prev_model_handle.id() {
                        commands.entity(entity).insert(effect.model_handle.clone());
                    }
                },
                Ok((None, Some(handle))) => {
                    if handle.id() == effect.prev_glass_handle.id() {
                        commands.entity(entity).insert(effect.glass_handle.clone());
                    }
                },
                _ => continue
            }
        }
        commands.entity(entity).insert(effect);
    }
    for (construction, effect) in set.p1().iter() {
        let progress = options.ease.calculate(construction.calculate(fraction));
        let mut material = effect_materials.get_mut(&effect.model_handle).unwrap();
        material.alpha_threshold = progress * (1.0 + material.dissolve_offset.x);
        let mut glass_material = materials.get_mut(&effect.glass_handle).unwrap();
        glass_material.base_color = glass_material.base_color.with_a(progress * effect.glass_alpha);
    }
    for (entity, children, effect) in set.p2().iter() {
        commands.entity(entity).remove::<ConstructionEffect>();
        for entity in query_descendants.iter_descendants(*children.first().unwrap()) {
            match query_mesh.get(entity) {
                Ok((Some(handle), None)) => {
                    if handle.id() == effect.model_handle.id() {
                        commands.entity(entity).insert(effect.prev_model_handle.clone());
                    }
                },
                Ok((None, Some(handle))) => {
                    if handle.id() == effect.glass_handle.id() {
                        commands.entity(entity).insert(effect.prev_glass_handle.clone());
                    }
                },
                _ => continue
            }
        }
    }
}