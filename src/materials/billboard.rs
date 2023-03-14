use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::render_resource::{ShaderRef, AsBindGroup};
use super::shared::{EffectMaterialKey, Billboard};

#[derive(TypeUuid, AsBindGroup, Clone, Default)]
#[uuid = "f25ffb20-22b3-4ad3-b8c6-ac176e9b3106"]
#[bind_group_data(EffectMaterialKey)]
pub struct BillboardEffectMaterial {
    #[uniform(0)]
    pub uv_transform: Vec4,
    #[uniform(0)]
    pub diffuse: Color,
    #[uniform(0)]
    pub alpha_threshold: f32,

    #[texture(1)]
    #[sampler(2)]
    pub diffuse_texture: Option<Handle<Image>>,

    pub billboard: Option<Billboard>,
    pub alpha_mode: AlphaMode,
}
impl From<&BillboardEffectMaterial> for EffectMaterialKey {
    fn from(value: &BillboardEffectMaterial) -> Self { Self {
        diffuse_map: value.diffuse_texture.is_some(),
        alpha_mask: value.alpha_threshold > 0.0,
        billboard: value.billboard,
        cull_mode: None, color_mask: true,
        ..Default::default()
    } }
}

impl Material for BillboardEffectMaterial {
    fn fragment_shader() -> ShaderRef { "shaders/default_billboard.wgsl".into() }
    fn vertex_shader() -> ShaderRef { "shaders/default_billboard.wgsl".into() }
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