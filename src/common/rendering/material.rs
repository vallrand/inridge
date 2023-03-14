use bevy::prelude::*;
use bevy::pbr::{MeshPipeline, MeshPipelineKey, MeshUniform};
use bevy::asset::{AddAsset, AssetServer, Handle, HandleUntyped, load_internal_asset};
use bevy::core_pipeline::core_3d::{Opaque3d, Transparent3d};
use bevy::ecs::{
    system::{
        lifetimeless::{Read, SRes},
        SystemParamItem,
    },
};
use bevy::reflect::TypeUuid;
use bevy::render::render_phase::{
    DrawFunctionId, AddRenderCommand, DrawFunctions, PhaseItem,
    RenderCommand, RenderCommandResult, RenderPhase, TrackedRenderPass
};
use bevy::render::render_resource::CachedRenderPipelineId;
use bevy::render::{
    extract_component::ExtractComponentPlugin,
    mesh::{Mesh, MeshVertexBufferLayout},
    render_asset::{RenderAssets},
    render_resource::{
        AsBindGroup, AsBindGroupError, BindGroup, BindGroupLayout, OwnedBindingResource,
        PipelineCache, RenderPipelineDescriptor, Shader, ShaderRef, SpecializedMeshPipeline,
        SpecializedMeshPipelineError, SpecializedMeshPipelines, VertexBufferLayout
    },
    renderer::RenderDevice,
    texture::FallbackImage,
    view::{ExtractedView, Msaa, VisibleEntities},
    RenderApp, RenderSet,
};
use std::hash::Hash;
use std::marker::PhantomData;
use bevy::ecs::system::{ReadOnlySystemParam};
use bevy::ecs::query::ReadOnlyWorldQuery;

use super::extract::{RenderAsset,ExtractAssetPlugin,PreparedRenderAssets,PrepareAssetError};
use super::instancing::*;

pub trait ShaderMaterial: AsBindGroup + Send + Sync + Clone + TypeUuid + Sized + 'static {
    const INSTANCED: bool = false;
    type Phase: PhaseItem + CreateItemBuilder;
    type DrawCommand: RenderCommand<Self::Phase> + Send + Sync + 'static;
    type Filter: ReadOnlyWorldQuery;

    fn vertex_shader() -> ShaderRef { ShaderRef::Default }
    fn fragment_shader() -> ShaderRef { ShaderRef::Default }

    #[inline] fn pipeline_key(&self) -> MeshPipelineKey { MeshPipelineKey::BLEND_OPAQUE }

    #[inline]
    fn depth_bias(&self) -> f32 { 0.0 }

    fn extract_layout(world: &mut World) -> Vec<BindGroupLayout> {
        let view_layout = world.resource::<bevy::pbr::MeshPipeline>().view_layout.clone();
        let mesh_layout = world.resource::<bevy::pbr::MeshPipeline>().mesh_layout.clone();
        let material_layout = Self::bind_group_layout(world.resource::<RenderDevice>());
        vec![view_layout, material_layout, mesh_layout]
    }

    fn specialize(
        _pipeline: &ShaderMaterialPipeline<Self>,
        descriptor: RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: (MeshPipelineKey, Self::Data),
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> { Ok(descriptor) }
}

const MESH_BINDINGS_HANDLE: HandleUntyped = HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 6152327284100772452);
const PBR_TEMPLATE_HANDLE: HandleUntyped = HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 6152327284100772453);
const LAYERED_TEMPLATE_HANDLE: HandleUntyped = HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 6152327284100772454);

pub struct ShaderMaterialPlugin<M: ShaderMaterial>(PhantomData<M>);
impl<M: ShaderMaterial> Default for ShaderMaterialPlugin<M> { fn default() -> Self { Self(Default::default()) } }
impl<M: ShaderMaterial> Plugin for ShaderMaterialPlugin<M>
where M::Data: PartialEq + Eq + Hash + Clone,
<M::DrawCommand as RenderCommand<M::Phase>>::Param: ReadOnlySystemParam {
    fn build(&self, app: &mut App) {
        load_internal_asset!(app, MESH_BINDINGS_HANDLE, "mesh_bindings.wgsl", Shader::from_wgsl);
        load_internal_asset!(app, PBR_TEMPLATE_HANDLE, "pbr_template.wgsl", Shader::from_wgsl);
        load_internal_asset!(app, LAYERED_TEMPLATE_HANDLE, "layered_template.wgsl", Shader::from_wgsl);

        app.add_asset::<M>().add_plugin(ExtractComponentPlugin::<Handle<M>>::extract_visible());
        app.add_plugin(ExtractAssetPlugin::<PreparedMaterial<M>>::default());
        app.sub_app_mut(RenderApp)
            .add_render_command::<M::Phase, M::DrawCommand>()
            .init_resource::<ShaderMaterialPipeline<M>>()
            .init_resource::<SpecializedMeshPipelines<ShaderMaterialPipeline<M>>>();
        if M::INSTANCED {
            app.sub_app_mut(RenderApp)
            .add_system(queue_instanced_meshes::<M>.in_set(RenderSet::Queue));
        } else {
            app.sub_app_mut(RenderApp)
            .add_system(queue_material_instances::<M>.in_set(RenderSet::Queue));
        }
    }
}

#[derive(Resource)]
pub struct ShaderMaterialPipeline<M: ShaderMaterial> {
    pub mesh_pipeline: MeshPipeline,
    pub layout_cache: Vec<BindGroupLayout>,
    pub material_layout: BindGroupLayout,
    pub vertex_shader: Option<Handle<Shader>>,
    pub fragment_shader: Option<Handle<Shader>>,
    pub instance_layout: Option<VertexBufferLayout>,
    marker: PhantomData<M>,
}

impl<M: ShaderMaterial> SpecializedMeshPipeline for ShaderMaterialPipeline<M>
where M::Data: PartialEq + Eq + Hash + Clone {
    type Key = (MeshPipelineKey, M::Data);
    fn specialize(
        &self, key: Self::Key, layout: &MeshVertexBufferLayout,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut descriptor = self.mesh_pipeline.specialize(key.0, layout)?;
        if let Some(vertex_shader) = &self.vertex_shader {
            descriptor.vertex.shader_defs.push("VERTEX".into());
            descriptor.vertex.shader = vertex_shader.clone();
        }
        if let Some(fragment_shader) = &self.fragment_shader {
            descriptor.fragment.as_mut().unwrap().shader_defs.push("FRAGMENT".into());
            descriptor.fragment.as_mut().unwrap().shader = fragment_shader.clone();
        }
        if let Some(layout) = self.instance_layout.as_ref() {
            descriptor.vertex.buffers.push(layout.clone());
            descriptor.vertex.shader_defs.push("INSTANCING".into());
            descriptor.fragment.as_mut().unwrap().shader_defs.push("INSTANCING".into());
        }
        descriptor.layout.insert(1, self.material_layout.clone());
        M::specialize(&self, descriptor, layout, key)
    }
}
impl<M: ShaderMaterial> FromWorld for ShaderMaterialPipeline<M> {
    fn from_world(world: &mut World) -> Self {
        let layout_cache = M::extract_layout(world);
        let asset_server = world.resource::<AssetServer>();
        let render_device = world.resource::<RenderDevice>();

        ShaderMaterialPipeline {
            layout_cache,
            mesh_pipeline: world.resource::<MeshPipeline>().clone(),
            material_layout: M::bind_group_layout(render_device),
            vertex_shader: match M::vertex_shader() {
                ShaderRef::Default => None,
                ShaderRef::Handle(handle) => Some(handle),
                ShaderRef::Path(path) => Some(asset_server.load(path)),
            },
            fragment_shader: match M::fragment_shader() {
                ShaderRef::Default => None,
                ShaderRef::Handle(handle) => Some(handle),
                ShaderRef::Path(path) => Some(asset_server.load(path)),
            },
            instance_layout: (if M::INSTANCED {
                Some(InstanceVertexAttributes::layout())
            } else { None }),
            marker: PhantomData,
        }
    }
}

pub struct SetMaterialBindGroup<M: ShaderMaterial, const I: usize>(PhantomData<M>);
impl<P: PhaseItem, M: ShaderMaterial, const I: usize> RenderCommand<P> for SetMaterialBindGroup<M, I> {
    type Param = SRes<PreparedRenderAssets<PreparedMaterial<M>>>;
    type ViewWorldQuery = ();
    type ItemWorldQuery = Read<Handle<M>>;

    #[inline]
    fn render<'w>(
        _item: &P, _view: (), material_handle: &'_ Handle<M>,
        materials: SystemParamItem<'w, '_, Self::Param>, pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(material) = materials.into_inner().get(material_handle) else { return RenderCommandResult::Failure };
        pass.set_bind_group(I, &material.bind_group, &[]);
        RenderCommandResult::Success
    }
}

pub trait CreateItemBuilder: PhaseItem {
    fn new(entity: Entity, draw_function: DrawFunctionId, pipeline: CachedRenderPipelineId, distance: f32) -> Self;
}
impl CreateItemBuilder for Transparent3d {
    fn new(entity: Entity, draw_function: DrawFunctionId, pipeline: CachedRenderPipelineId, distance: f32) -> Self {
        Self { entity, draw_function, pipeline, distance }
    }
}
impl CreateItemBuilder for Opaque3d {
    fn new(entity: Entity, draw_function: DrawFunctionId, pipeline: CachedRenderPipelineId, distance: f32) -> Self {
        Self { entity, draw_function, pipeline, distance }
    }
}

pub fn queue_material_instances<M: ShaderMaterial>(
    draw_functions: Res<DrawFunctions<M::Phase>>,
    specialize_pipeline: Res<ShaderMaterialPipeline<M>>,
    mut pipelines: ResMut<SpecializedMeshPipelines<ShaderMaterialPipeline<M>>>,
    pipeline_cache: Res<PipelineCache>,

    msaa: Res<Msaa>,
    render_meshes: Res<RenderAssets<Mesh>>,
    render_materials: Res<PreparedRenderAssets<PreparedMaterial<M>>>,
    instance_query: Query<(&Handle<M>, &Handle<Mesh>, &MeshUniform), M::Filter>,
    mut views: Query<(&ExtractedView, &VisibleEntities, &mut RenderPhase<M::Phase>)>,
) where M::Data: PartialEq + Eq + Hash + Clone {
    let draw_function = draw_functions.read().id::<M::DrawCommand>();

    for (view, visible_entities, mut render_phase) in &mut views {
        let view_key = MeshPipelineKey::from_msaa_samples(msaa.samples()) | MeshPipelineKey::from_hdr(view.hdr);

        let rangefinder = view.rangefinder3d();
        for visible_entity in &visible_entities.entities {
            let Ok((material_handle, mesh_handle, mesh_uniform)) =
                instance_query.get(*visible_entity) else { continue };
            
            let (Some(mesh), Some(material)) = (
                render_meshes.get(mesh_handle),
                render_materials.get(material_handle),
            ) else { continue };

            let Ok(pipeline_id) = pipelines.specialize(
                &pipeline_cache, &specialize_pipeline, (
                    material.pipeline_key | view_key | MeshPipelineKey::from_primitive_topology(mesh.primitive_topology),
                    material.key.clone()
                ), &mesh.layout,
            ) else { continue };

            let distance = rangefinder.distance(&mesh_uniform.transform) + material.depth_bias;
            render_phase.add(M::Phase::new(*visible_entity, draw_function, pipeline_id, distance));            
        }
    }
}

pub struct PreparedMaterial<T: ShaderMaterial> {
    pub bindings: Vec<OwnedBindingResource>,
    pub bind_group: BindGroup,
    pub key: T::Data,
    pub pipeline_key: MeshPipelineKey,
    pub depth_bias: f32,
}

impl<M: ShaderMaterial> RenderAsset for PreparedMaterial<M> {
    type SourceAsset = M;
    type ExtractedAsset = M;
    type Param = (
        SRes<RenderDevice>,
        SRes<RenderAssets<Image>>,
        SRes<FallbackImage>,
        SRes<ShaderMaterialPipeline<M>>
    );
    fn extract_asset(source_asset: &Self::SourceAsset) -> Self::ExtractedAsset { source_asset.clone() }
    fn prepare_asset(
        extracted_asset: Self::ExtractedAsset,
        (render_device, images, fallback_image, pipeline): &mut SystemParamItem<Self::Param>,
    ) -> Result<Self, PrepareAssetError<Self::ExtractedAsset>> {
        match extracted_asset.as_bind_group(
            &pipeline.material_layout, render_device, images, fallback_image,
        ) {
            Err(AsBindGroupError::RetryNextUpdate) => Err(PrepareAssetError::RetryNextUpdate(extracted_asset)),
            Ok(prepared) => Ok(PreparedMaterial {
                bindings: prepared.bindings,
                bind_group: prepared.bind_group,
                key: prepared.data,
                pipeline_key: extracted_asset.pipeline_key(),
                depth_bias: extracted_asset.depth_bias(),
            })
        }
    }
}




