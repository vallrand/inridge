use bevy::prelude::*;
use bevy::utils::HashMap;
use crate::common::loader::AssetBundle;
use crate::materials::{MatterEffectMaterial, ModelEffectLayeredMaterial};
use crate::scene::{ModelAssetBundle, UnitBlueprint};
use crate::logic::Agent;

#[derive(Resource, Deref, DerefMut, Clone, Default)]
pub struct MembershipSettings(pub HashMap<Agent, (
    Handle<ModelEffectLayeredMaterial>,
    Handle<MatterEffectMaterial>,
)>);

pub fn setup_membership_materials(
    mut membership_settings: ResMut<MembershipSettings>,
    mut materials_matter: ResMut<Assets<MatterEffectMaterial>>,
    mut materials: ResMut<Assets<ModelEffectLayeredMaterial>>,
    model_bundle: Res<AssetBundle<ModelAssetBundle>>,
){
    membership_settings.insert(Agent::Player, (
        materials.add(ModelEffectLayeredMaterial{
            albedo: model_bundle.albedo.clone(), normal: model_bundle.normal.clone(), rma: model_bundle.rma.clone(),
            color_shift: Color::WHITE,
            emission: 24.0, uv_transform: Vec4::new(0.0, 0.0, 1.0, 1.0),
            scanline_color: Color::rgb(0.6,0.0,0.1),
            scanline_width: Vec4::new(0.05, 0.4, 1.6, -0.5),
            damage: true, dissolve: false,
            noise_domain: Vec3::splat(4.0),
            dissolve_color: Color::rgb_linear(2.8, 3.6, 1.8),
            dissolve_plane: Vec4::ZERO,
            dissolve_offset: Vec2::new(0.4, 0.4),
            ..Default::default()
        }),
        materials_matter.add(MatterEffectMaterial {
            diffuse: Color::rgb(0.2,0.0,0.2),
            emissive: Color::rgb_linear(20.0, 26.0, 5.2),
            noise_domain: Vec3::splat(8.0),
            ..Default::default()
        })
    ));
    membership_settings.insert(Agent::AI(1), (
        materials.add(ModelEffectLayeredMaterial{
            albedo: model_bundle.albedo.clone(), normal: model_bundle.normal.clone(), rma: model_bundle.rma.clone(),
            color_shift: Color::BLACK,
            emission: 24.0, uv_transform: Vec4::new(0.0, 0.0, 1.0, 1.0),
            scanline_color: Color::rgb(0.6,0.0,0.1),
            scanline_width: Vec4::new(0.05, 0.4, 1.6, -0.5),
            damage: true, dissolve: false,
            noise_domain: Vec3::splat(4.0),
            dissolve_color: Color::rgb_linear(3.6, 2.8, 1.8),
            dissolve_plane: Vec4::ZERO,
            dissolve_offset: Vec2::new(0.4, 0.4),
            ..Default::default()
        }),
        materials_matter.add(MatterEffectMaterial {
            diffuse: Color::rgb(0.2,0.0,0.2),
            emissive: Color::rgb_linear(24.0, 8.0, 14.0),
            noise_domain: Vec3::splat(8.0),
            ..Default::default()
        })
    ));
}

pub fn apply_unit_membership(
    membership_settings: Res<MembershipSettings>,
    model_bundle: Res<AssetBundle<ModelAssetBundle>>,
    
    query_unit: Query<&Agent, With<Handle<UnitBlueprint>>>,
    query_scene: Query<(Entity, &Parent), Changed<bevy::scene::SceneInstance>>,
    children: Query<&Children>,

    mut query_material_matter: Query<&mut Handle<MatterEffectMaterial>>,
    mut query_material: Query<&mut Handle<ModelEffectLayeredMaterial>>,
){
    for (entity, parent) in query_scene.iter() {
        let Ok(agent) = query_unit.get(parent.get()) else { continue };
        let Some(handles) = membership_settings.get(agent) else { continue };
        for entity in children.iter_descendants(entity) {
            if let Ok(mut material) = query_material.get_mut(entity) {
                if material.id() == model_bundle.material.id() {
                    *material = handles.0.clone();
                }
            }
            if let Ok(mut material) = query_material_matter.get_mut(entity) {
                if material.id() == model_bundle.matter_material.id() {
                    *material = handles.1.clone();
                }
            }
        }
    }
}