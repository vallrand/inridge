use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::reflect::TypeUuid;
use crate::common::rendering::{ShaderMaterial,SetMaterialBindGroup,DrawMeshInstanced};

#[derive(AsBindGroup, TypeUuid, Clone, Default, Debug)]
#[uuid = "0a8f7aa6-de5d-4280-9d15-4971e2521d30"]
pub struct LayeredInstancedStandardMaterial {
    #[texture(0, dimension = "2d_array")]
    #[sampler(1)]
    pub albedo: Handle<Image>,
    #[texture(2, dimension = "2d_array")]
    #[sampler(3)]
    pub normal: Handle<Image>,
    #[texture(4, dimension = "2d_array")]
    #[sampler(5)]
    pub rma: Handle<Image>,
    pub cull_mode: Option<bevy::render::render_resource::Face>,
    #[uniform(6)]
    pub uv_transform: Vec4,
    #[uniform(6)]
    pub emission: f32,
}

impl Material for LayeredInstancedStandardMaterial {
    fn fragment_shader() -> ShaderRef {"shaders/default_layered.wgsl".into()}
}
impl ShaderMaterial for LayeredInstancedStandardMaterial {
    const INSTANCED: bool = true;
    type Phase = bevy::core_pipeline::core_3d::Opaque3d;
    type DrawCommand = (
        bevy::render::render_phase::SetItemPipeline,
        bevy::pbr::SetMeshViewBindGroup<0>,
        SetMaterialBindGroup<Self, 1>,
        DrawMeshInstanced
    );
    type Filter = ();
    fn fragment_shader() -> ShaderRef {"shaders/default_layered.wgsl".into()}
    fn vertex_shader() -> ShaderRef {"shaders/default_layered.wgsl".into()}
    fn specialize(
        _pipeline: &crate::common::rendering::ShaderMaterialPipeline<Self>,
        mut descriptor: bevy::render::render_resource::RenderPipelineDescriptor,
        _layout: &bevy::render::mesh::MeshVertexBufferLayout,
        _key: (bevy::pbr::MeshPipelineKey, Self::Data),
    ) -> Result<bevy::render::render_resource::RenderPipelineDescriptor, bevy::render::render_resource::SpecializedMeshPipelineError> {
        descriptor.layout = vec![
            descriptor.layout[0].clone(),
            descriptor.layout[1].clone(),
        ];
        Ok(descriptor)
    }
}

#[derive(AsBindGroup, TypeUuid, Clone, Default, Debug)]
#[uuid = "326d151d-69cc-4ed1-9e8c-e674aadc221f"]
pub struct TerrainLayeredMaterial {
    #[texture(0, dimension = "2d_array")]
    #[sampler(1)]
    pub albedo: Handle<Image>,
    #[texture(2, dimension = "2d_array")]
    #[sampler(3)]
    pub normal: Handle<Image>,
    #[texture(4, dimension = "2d_array")]
    #[sampler(5)]
    pub rma: Handle<Image>,

    #[uniform(6)]
    pub emission: f32,
    #[uniform(6)]
    pub uv_scale: Vec2,
    #[uniform(6)]
    pub border_width: f32,
}

impl Material for TerrainLayeredMaterial {
    fn fragment_shader() -> ShaderRef {"shaders/default_terrain.wgsl".into()}
}