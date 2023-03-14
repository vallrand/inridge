use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::render_resource::{ShaderRef, AsBindGroup};
use super::shared::{EffectMaterialKey, Billboard};

#[derive(TypeUuid, AsBindGroup, Clone, Default)]
#[uuid = "3e6c0494-0a95-4a45-ab55-9a60f744bea1"]
#[bind_group_data(EffectMaterialKey)]
pub struct ExplosionMatterial {
    #[uniform(0)]
    pub uv_transform: Vec4,
    #[uniform(0)]
    pub color: Color,
}
impl From<&ExplosionMatterial> for EffectMaterialKey {
    fn from(_value: &ExplosionMatterial) -> Self { Self {
        billboard: Some(Billboard::SphericalScreen),
        color_mask: true,
        ..Default::default()
    } }
}

impl Material for ExplosionMatterial {
    fn fragment_shader() -> ShaderRef { "shaders/effect_explosion.wgsl".into() }
    fn vertex_shader() -> ShaderRef { "shaders/default_billboard.wgsl".into() }
    fn alpha_mode(&self) -> AlphaMode { AlphaMode::Blend }
    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline<Self>,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        _layout: &bevy::render::mesh::MeshVertexBufferLayout,
        key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        key.bind_group_data.apply(descriptor);
        Ok(())
    }
}