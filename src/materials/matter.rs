use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};

#[derive(AsBindGroup, TypeUuid, Clone, Default, Debug)]
#[uuid = "8bbd711d-126d-4490-a466-b59909817584"]
pub struct MatterEffectMaterial {
    #[uniform(0)]
    pub diffuse: Color,
    #[uniform(0)]
    pub emissive: Color,
    #[uniform(0)]
    pub noise_domain: Vec3,
    #[uniform(0)]
    pub flags: u32,
}
impl Material for MatterEffectMaterial {
    fn fragment_shader() -> ShaderRef { "shaders/effect_matter.wgsl".into() }
}