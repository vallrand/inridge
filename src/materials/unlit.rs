use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::render_resource::{
    AsBindGroup,RenderPipelineDescriptor,SpecializedMeshPipelineError,ShaderRef,BindGroupLayout,Face
};
use bevy::render::renderer::RenderDevice;
use bevy::render::mesh::MeshVertexBufferLayout;
use bevy::core_pipeline::core_3d::Transparent3d;
use bevy::render::render_phase::{SetItemPipeline};
use bevy::pbr::{DrawMesh, SetMeshBindGroup, SetMeshViewBindGroup, MeshPipelineKey};
use crate::common::rendering::{ShaderMaterialPipeline,ShaderMaterial,SetMaterialBindGroup,};
use super::shared::EffectMaterialKey;

#[derive(AsBindGroup, TypeUuid, Clone, Default, Debug)]
#[uuid = "28438702-70e6-441f-af01-965ecc4683cd"]
#[bind_group_data(EffectMaterialKey)]
pub struct UnlitMaterial {
    #[uniform(0)]
    pub color: Color,
    #[uniform(0)]
    pub fresnel_mask: f32,
    #[uniform(0)]
    pub depth_mask: f32,

    #[texture(1)]
    #[sampler(2)]
    pub diffuse: Option<Handle<Image>>,

    pub depth_bias: i32,
    pub double_sided: bool,
}

impl From<&UnlitMaterial> for EffectMaterialKey {
    fn from(material: &UnlitMaterial) -> Self { Self {
        diffuse_map: material.diffuse.is_some(),
        fresnel_mask: material.fresnel_mask > 0.0,
        depth_mask: material.depth_mask > 0.0,
        depth_bias: material.depth_bias,
        cull_mode: if material.double_sided { None }else{ Some(Face::Back) },
        ..Default::default()
    } }
}

impl ShaderMaterial for UnlitMaterial {
    type Phase = Transparent3d;
    type DrawCommand = (
        SetItemPipeline,
        SetMeshViewBindGroup<0>,
        SetMaterialBindGroup<Self, 1>,
        SetMeshBindGroup<2>,
        DrawMesh,
    );
    type Filter = ();

    fn extract_layout(world: &mut World) -> Vec<BindGroupLayout> {
        let view_layout = world.resource::<bevy::pbr::MeshPipeline>().view_layout.clone();
        let mesh_layout = world.resource::<bevy::pbr::MeshPipeline>().mesh_layout.clone();
        let material_layout = Self::bind_group_layout(world.resource::<RenderDevice>());
        vec![view_layout, material_layout, mesh_layout]
    }

    fn fragment_shader() -> ShaderRef {"shaders/default_unlit.wgsl".into()}
    fn vertex_shader() -> ShaderRef {"shaders/default_unlit.wgsl".into()}
    fn pipeline_key(&self) -> MeshPipelineKey { MeshPipelineKey::BLEND_ALPHA }
    fn depth_bias(&self) -> f32 { self.depth_bias as f32 }

    fn specialize(
        pipeline: &ShaderMaterialPipeline<Self>,
        mut descriptor: RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        key: (MeshPipelineKey, Self::Data),
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        use bevy::render::render_resource::*;
        key.1.apply(&mut descriptor);

        Ok(RenderPipelineDescriptor {
            layout: vec![
                descriptor.layout[0].clone(),
                pipeline.layout_cache[1].clone(),
                descriptor.layout[2].clone(),
            ],
            depth_stencil: Some(DepthStencilState {
                depth_compare: CompareFunction::GreaterEqual,
                depth_write_enabled: false,
                bias: DepthBiasState{
                    constant: key.1.depth_bias,
                    slope_scale: 0.0,
                    clamp: -1.0,
                },
                ..descriptor.depth_stencil.unwrap()
            }),
            ..descriptor
        })
    }
}