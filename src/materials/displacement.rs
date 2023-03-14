use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType, BindGroupEntry, ShaderRef, TextureFormat};
use bevy::render::extract_component::ExtractComponent;
use bevy::ecs::system::lifetimeless::Read;
use bevy::ecs::query::QueryItem;
use crate::extensions::{PostProcess, RenderTexturePass, ViewRenderTexture, RenderTexturePhase};
use super::shared::EffectMaterialKey;

#[derive(Component, ShaderType, ExtractComponent, Clone, Copy, Default)]
pub struct DisplacementSettings {
    pub intensity: f32,
    pub chromatic_aberration: f32,
}

impl RenderTexturePass for DisplacementSettings {
    const KEY: &'static str = "displacement_pass";
    const TEXTURE_FORMAT: TextureFormat = TextureFormat::Rgba16Float;
}
use bevy::render::render_resource::{TextureViewDimension, BindGroupLayoutEntry, BindingResource, TextureSampleType, BindingType, ShaderStages};
impl PostProcess for DisplacementSettings {
    const KEY: &'static str = "displacement";
    type ViewQuery = Read<ViewRenderTexture<Self>>;
    fn fragment_shader() -> ShaderRef { "shaders/post_effect_displacement.wgsl".into() }
    fn extract_bindings(item: QueryItem<'_, Self::ViewQuery>) -> Option<Vec<BindGroupEntry>> {
        Some(vec![BindGroupEntry {
            binding: 3, resource: BindingResource::TextureView(&item.render_texture.default_view),
        }])
    }
    fn bind_group_layout(entries: &mut Vec<BindGroupLayoutEntry>) {
        entries.push(BindGroupLayoutEntry {
            binding: 3,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Texture {
                sample_type: TextureSampleType::Float { filterable: true },
                view_dimension: TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        });
    }
}

use bevy::reflect::TypeUuid;
use crate::common::rendering::{ShaderMaterial, SetMaterialBindGroup};
use bevy::render::render_phase::SetItemPipeline;
use bevy::pbr::{DrawMesh, SetMeshBindGroup, SetMeshViewBindGroup, MeshPipelineKey};

#[derive(AsBindGroup, TypeUuid, Clone, Default)]
#[uuid = "24a44e86-daf5-445b-b9cc-a47c0faebb0e"]
#[bind_group_data(EffectMaterialKey)]
pub struct DisplacementMaterial {
    #[uniform(0)]
    pub displacement: f32,
    #[uniform(0)]
    pub chromatic_aberration: f32,
    #[texture(1)]
    #[sampler(2)]
    pub displacement_map: Option<Handle<Image>>,
    pub fresnel: bool,
    pub mask: bool,
}

impl From<&DisplacementMaterial> for EffectMaterialKey {
    fn from(value: &DisplacementMaterial) -> Self { Self {
        displacement_map: value.displacement_map.is_some(),
        fresnel_mask: value.fresnel,
        color_mask: value.mask,
        ..Default::default()
    } }
}

impl ShaderMaterial for DisplacementMaterial {
    const INSTANCED: bool = false;
    type Phase = RenderTexturePhase<DisplacementSettings>;
    type DrawCommand = (
        SetItemPipeline,
        SetMeshViewBindGroup<0>,
        SetMaterialBindGroup<Self, 1>,
        SetMeshBindGroup<2>,
        DrawMesh,
    );
    type Filter = ();
    fn pipeline_key(&self) -> MeshPipelineKey { MeshPipelineKey::BLEND_OPAQUE }
    fn fragment_shader() -> ShaderRef {"shaders/default_displacement.wgsl".into()}
    fn vertex_shader() -> ShaderRef {"shaders/default_displacement.wgsl".into()}
    fn specialize(
        _pipeline: &crate::common::rendering::ShaderMaterialPipeline<Self>,
        mut descriptor: bevy::render::render_resource::RenderPipelineDescriptor,
        _layout: &bevy::render::mesh::MeshVertexBufferLayout,
        key: (MeshPipelineKey, Self::Data),
    ) -> Result<bevy::render::render_resource::RenderPipelineDescriptor, bevy::render::render_resource::SpecializedMeshPipelineError> {
        key.1.apply(&mut descriptor);
        Ok(descriptor)
    }
}