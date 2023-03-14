use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use super::shared::EffectMaterialKey;

#[derive(TypeUuid, AsBindGroup, Clone, Default, Debug)]
#[uuid = "58966fcd-0b10-4744-bb05-34e80cbc702a"]
#[bind_group_data(EffectMaterialKey)]
pub struct BarrierEffectMaterial {
    #[uniform(0)]
    pub color: Color,
    #[uniform(0)]
    pub fresnel_mask: f32,
}

impl From<&BarrierEffectMaterial> for EffectMaterialKey {
    fn from(_value: &BarrierEffectMaterial) -> Self { Self {
        fresnel_mask: true,
        cull_mode: None,
        ..Default::default()
    } }
}


impl Material for BarrierEffectMaterial {
    fn fragment_shader() -> ShaderRef { "shaders/effect_barrier.wgsl".into() }
    fn alpha_mode(&self) -> AlphaMode { AlphaMode::Premultiplied }
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