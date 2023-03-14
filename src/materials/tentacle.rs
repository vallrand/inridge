use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::render_resource::{AsBindGroup,ShaderRef};

#[derive(TypeUuid, AsBindGroup, Clone, Default, Debug)]
#[uuid = "d0843a0c-a863-4e7c-a5d1-36c85776ab52"]
pub struct TentacleEffectMaterial {
    #[uniform(0)]
    pub color: Color,
    #[uniform(0)]
    pub uv_transform: Vec4,
    #[uniform(0)]
    pub reflectance: f32,
    #[uniform(0)]
    pub metallic: f32,
    #[uniform(0)]
    pub roughness: f32,
}

impl Material for TentacleEffectMaterial {
    fn vertex_shader() -> ShaderRef { "shaders/effect_tentacle.wgsl".into() }
    fn fragment_shader() -> ShaderRef { "shaders/effect_tentacle.wgsl".into() }
}