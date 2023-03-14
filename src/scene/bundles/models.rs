use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy::gltf::Gltf;
use bevy::render::primitives::Aabb;
use crate::common::loader::{AssetBundleList, ScopedAssetServer};
use crate::extensions::{clone_entities, replace_components};
use crate::common::spatial::aabb::AABB;
use crate::materials::{ColorUniform, LayeredInstancedStandardMaterial};
use crate::materials::{ModelEffectLayeredMaterial, MatterEffectMaterial};
use super::super::UnitBlueprint;
use super::texture::TextureSamplerOptions;

pub struct ModelAssetBundle {
    pub albedo: Handle<Image>,
    pub normal: Handle<Image>,
    pub rma: Handle<Image>,
    pub models: Handle<Gltf>,
    pub extracted: HashMap<String, (Handle<Scene>, Aabb)>,
    pub animations: HashMap<String, Handle<AnimationClip>>,

    pub material: Handle<ModelEffectLayeredMaterial>,
    pub static_material: Handle<LayeredInstancedStandardMaterial>,
    pub glass_material: Handle<StandardMaterial>,
    pub matter_material: Handle<MatterEffectMaterial>,
}
impl ModelAssetBundle {
    pub fn model_from(&self, blueprint: &UnitBlueprint, commands: &mut Commands) -> Entity {
        let (model_scene, model_aabb) = self.extracted.get(&blueprint.model).unwrap();
        let model = commands.spawn(SceneBundle {
            scene: model_scene.clone(), ..Default::default()
        }).insert(model_aabb.clone()).id();
        if let Some(component) = blueprint.animation.as_ref() {
            commands.entity(model).insert(component.clone());
        }
        model
    }
}
impl AssetBundleList for ModelAssetBundle {
    fn from_asset_server(asset_server: &ScopedAssetServer) -> Self { Self {
        albedo: asset_server.load("textures/model_albedo.png"),
        normal: asset_server.load("textures/model_normal.png"),
        rma: asset_server.load("textures/model_rma.png"),
        models: asset_server.load("models.gltf"),
        extracted: HashMap::new(),
        animations: HashMap::new(),
        material: Default::default(),
        static_material: Default::default(),
        glass_material: Default::default(),
        matter_material: Default::default(),
    } }
    fn prepare(&mut self, world: &mut World) {
        let mut images = world.resource_mut::<Assets<Image>>();
                
        let array_layers = 8;
        images.get_mut(&self.albedo).unwrap().reinterpret_stacked_2d_as_array(array_layers);
        images.get_mut(&self.normal).unwrap().reinterpret_stacked_2d_as_array(array_layers);
        images.get_mut(&self.rma).unwrap().reinterpret_stacked_2d_as_array(array_layers);

        TextureSamplerOptions::SRGBA.apply(images.get_mut(&self.albedo).unwrap());
        TextureSamplerOptions::DEFAULT.apply(images.get_mut(&self.normal).unwrap());
        TextureSamplerOptions::DEFAULT.apply(images.get_mut(&self.rma).unwrap());

        self.static_material = world.resource_mut::<Assets<LayeredInstancedStandardMaterial>>()
        .add(LayeredInstancedStandardMaterial{
            albedo: self.albedo.clone(), normal: self.normal.clone(), rma: self.rma.clone(),
            emission: 4.0, uv_transform: Vec4::new(0.0, 0.0, 1.0, 1.0), ..Default::default()
        });
        self.material = world.resource_mut::<Assets<ModelEffectLayeredMaterial>>()
        .add(ModelEffectLayeredMaterial{
            albedo: self.albedo.clone(), normal: self.normal.clone(), rma: self.rma.clone(),
            emission: 24.0, uv_transform: Vec4::new(0.0, 0.0, 1.0, 1.0),
            scanline_color: Color::rgb(0.6,0.0,0.1),
            scanline_width: Vec4::new(0.05, 0.4, 1.6, -0.5),
            damage: true,
            noise_domain: Vec3::splat(4.0), ..Default::default()
        });
        self.matter_material = world.resource_mut::<Assets<MatterEffectMaterial>>().add(MatterEffectMaterial::default());
                
        world.resource_scope(|world, mut scene_assets: Mut<Assets<Scene>>|{
            let gltf = world.resource::<Assets<bevy::gltf::Gltf>>().get(&self.models).unwrap();
            self.animations = gltf.named_animations.clone();
            let default_scene = gltf.default_scene.as_ref().unwrap();

            self.glass_material = gltf.named_materials.get("glass").unwrap().clone();

            let registry = world.resource::<AppTypeRegistry>().clone();
            let mut extracted: Vec<(String, Scene)> = Vec::new();
            if let Some(scene) = scene_assets.get_mut(&default_scene) {
                Schedule::new()
                .add_system(bevy::transform::systems::sync_simple_transforms)
                .add_system(bevy::transform::systems::propagate_transforms)
                .run(&mut scene.world);

                replace_components(scene, gltf.named_materials.get("fiber".into()).unwrap(), (
                    self.material.clone(),
                    ColorUniform::from(Color::BLACK),
                ));
                replace_components(scene, gltf.named_materials.get("matter".into()).unwrap(), self.matter_material.clone());

                let collection = scene.world.query_filtered::<
                    &Children, (Without<Name>, Without<Parent>)
                >().single(&scene.world).to_vec();
                for entity in collection.into_iter() {
                    scene.world.entity_mut(entity).remove::<Parent>();
                    let name = scene.world.get::<Name>(entity).unwrap();
                    let mut subworld = World::default();
                    subworld.insert_resource(registry.clone());
                    if let Ok (_) = clone_entities(&scene.world, &mut subworld, |mut node|{
                        loop {
                            if node == entity { break true; }
                            if let Some(parent) = scene.world.get::<Parent>(node) {
                                node = parent.get();
                            } else {
                                break false;
                            }
                        }
                    }) {
                        extracted.push((name.into(), Scene::new(subworld)));
                    }
                }
            }

            let meshes = world.resource::<Assets<Mesh>>();
            for (key, mut scene) in extracted.into_iter() {
                let mut combined_aabb: AABB = AABB::default();
                for (handle, transform) in scene.world.query::<(&Handle<Mesh>, &GlobalTransform)>().iter(&scene.world) {
                    let Some(mut aabb) = meshes.get(handle).and_then(|mesh|mesh.compute_aabb()) else { continue };
                    aabb.center = transform.affine().transform_point3a(aabb.center);
                    combined_aabb += AABB::from_min_max(aabb.min(), aabb.max());
                }
                let combined_aabb = Aabb::from_min_max(combined_aabb.min.into(), combined_aabb.max.into());
                self.extracted.insert(key, (scene_assets.add(scene), combined_aabb));
            }
        });
    }
}