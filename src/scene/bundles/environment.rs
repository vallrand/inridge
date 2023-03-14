use bevy::prelude::*;
use crate::common::loader::{AssetBundleList, ScopedAssetServer};
use crate::materials::TerrainLayeredMaterial;
use super::texture::TextureSamplerOptions;

pub struct EnvironmentAssetBundle {
    albedo: Handle<Image>,
    normal: Handle<Image>,
    rma: Handle<Image>,
    pub terrain_material: Handle<TerrainLayeredMaterial>,
}
impl AssetBundleList for EnvironmentAssetBundle {
    fn from_asset_server(asset_server: &ScopedAssetServer) -> Self { Self {
        albedo: asset_server.load("textures/terrain_albedo.png"),
        normal: asset_server.load("textures/terrain_normal.png"),
        rma: asset_server.load("textures/terrain_rma.png"),
        terrain_material: Default::default(),
    } }
    fn prepare(&mut self, world: &mut World) {
        let mut images = world.resource_mut::<Assets<Image>>();
        
        let array_layers = 4;
        images.get_mut(&self.albedo).unwrap().reinterpret_stacked_2d_as_array(array_layers);
        images.get_mut(&self.normal).unwrap().reinterpret_stacked_2d_as_array(array_layers);
        images.get_mut(&self.rma).unwrap().reinterpret_stacked_2d_as_array(array_layers);

        TextureSamplerOptions::SRGBA.apply(images.get_mut(&self.albedo).unwrap());
        TextureSamplerOptions::DEFAULT.apply(images.get_mut(&self.normal).unwrap());
        TextureSamplerOptions::DEFAULT.apply(images.get_mut(&self.rma).unwrap());

        self.terrain_material = world.resource_mut::<Assets<TerrainLayeredMaterial>>()
        .add(TerrainLayeredMaterial{
            albedo: self.albedo.clone(), normal: self.normal.clone(), rma: self.rma.clone(),
            emission: 16.0,
            border_width: 0.02,
            uv_scale: Vec2::splat(4.0),
        });
    }
}
