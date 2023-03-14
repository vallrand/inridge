use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};

#[derive(AsBindGroup, TypeUuid, Clone, Default, Debug)]
#[uuid = "6b169d73-3e8f-435f-881c-779e6d85a882"]
pub struct ReconstructEffectMaterial {
    #[uniform(0)]
    pub color: Color,
    #[uniform(0)]
    pub glow_color: Color,
    #[uniform(0)]
    pub domain: Vec2,
    #[uniform(0)]
    pub vertical_fade: Vec2,
    #[uniform(0)]
    pub threshold: f32,
    pub alpha_mode: AlphaMode,
}
impl Material for ReconstructEffectMaterial {
    fn fragment_shader() -> ShaderRef { "shaders/effect_reconstruct.wgsl".into() }
    fn alpha_mode(&self) -> AlphaMode { self.alpha_mode }
    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline<Self>,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        _layout: &bevy::render::mesh::MeshVertexBufferLayout,
        _key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}