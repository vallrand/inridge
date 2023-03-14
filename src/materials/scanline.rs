use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::render_resource::{AsBindGroup, ShaderRef, Face};
use super::shared::EffectMaterialKey;

#[derive(AsBindGroup, TypeUuid, Clone, Default, Debug)]
#[uuid = "6ed91206-99ce-44b2-9c71-f0972573baf9"]
#[bind_group_data(EffectMaterialKey)]
pub struct ScanlineEffectMaterial {
    #[uniform(0)]
    pub color: Color,
    #[uniform(0)]
    pub uv_transform: Vec4,
    #[uniform(0)]
    pub vertical_fade: Vec2,
    #[uniform(0)]
    pub line_width: Vec4,
    
    pub alpha_mode: AlphaMode,
    pub cull_mode: Option<Face>,
}
impl Material for ScanlineEffectMaterial {
    fn vertex_shader() -> ShaderRef { "shaders/effect_scanline.wgsl".into() }
    fn fragment_shader() -> ShaderRef { "shaders/effect_scanline.wgsl".into() }
    fn alpha_mode(&self) -> AlphaMode { self.alpha_mode }
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

impl From<&ScanlineEffectMaterial> for EffectMaterialKey {
    fn from(value: &ScanlineEffectMaterial) -> Self { Self {
        cull_mode: value.cull_mode, color_mask: true,
        ..Default::default()
    } }
}
