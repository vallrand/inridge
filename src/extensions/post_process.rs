use std::marker::PhantomData;
use bevy::prelude::*;
use bevy::render::extract_component::{ComponentUniforms, ExtractComponent, ExtractComponentPlugin, UniformComponentPlugin};
use bevy::render::render_graph::{Node, NodeRunError, RenderGraph, RenderGraphContext, SlotInfo, SlotType};
use bevy::render::render_resource::{
    BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, CachedRenderPipelineId,
    ColorTargetState, ColorWrites, FragmentState, MultisampleState, Operations,
    PipelineCache, PrimitiveState, RenderPassColorAttachment, RenderPassDescriptor,
    RenderPipelineDescriptor, Sampler, SamplerBindingType, SamplerDescriptor, ShaderStages,
    ShaderType, TextureFormat, TextureSampleType, TextureViewDimension, ShaderRef, encase::internal::WriteInto
};
use bevy::core_pipeline::core_3d;
use bevy::core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state;
use bevy::ecs::query::{QueryItem, WorldQuery, ReadOnlyWorldQuery};
use bevy:: render::{
    renderer::{RenderContext, RenderDevice},
    view::{ExtractedView, ViewTarget},
    texture::BevyDefault,
    RenderApp,
};
pub trait PostProcess: Component + ShaderType + WriteInto + ExtractComponent + Clone + Sync + Send + 'static {
    const INPUT_NODE: &'static str = core_3d::graph::node::TONEMAPPING;
    const OUTPUT_NODE: &'static str = core_3d::graph::node::END_MAIN_PASS_POST_PROCESSING;
    const GRAPH_KEY: &'static str = core_3d::graph::NAME;
    const KEY: &'static str;

    type ViewQuery: WorldQuery + ReadOnlyWorldQuery;
    fn fragment_shader() -> ShaderRef { ShaderRef::Default }
    fn extract_bindings(_item: QueryItem<'_, Self::ViewQuery>) -> Option<Vec<BindGroupEntry>> { None }
    fn bind_group_layout(_entries: &mut Vec<BindGroupLayoutEntry>){}
}

pub struct PostProcessPlugin<T: PostProcess>(PhantomData<T>);
impl<T: PostProcess> Default for PostProcessPlugin<T> { fn default() -> Self { Self(PhantomData) } }
impl<T: PostProcess> Plugin for PostProcessPlugin<T> {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(ExtractComponentPlugin::<T>::default())
            .add_plugin(UniformComponentPlugin::<T>::default());

        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<PostProcessPipeline::<T>>();

        let node = PostProcessNode::<T>::from_world(&mut render_app.world);
        let mut graph = render_app.world.resource_mut::<RenderGraph>();
        let subgraph = graph.get_sub_graph_mut(T::GRAPH_KEY).unwrap();
        subgraph.add_node(T::KEY, node);
        subgraph.add_slot_edge(
            subgraph.input_node().id, core_3d::graph::input::VIEW_ENTITY,
            T::KEY, PostProcessNode::<T>::IN_VIEW,
        );
        subgraph.add_node_edge(T::INPUT_NODE, T::KEY);
        subgraph.add_node_edge(T::KEY, T::OUTPUT_NODE);
    }
}

pub struct PostProcessNode<T: PostProcess> {
    query: QueryState<(&'static ViewTarget, T::ViewQuery), With<ExtractedView>>,
    marker: PhantomData<T>,
}
impl<T: PostProcess> PostProcessNode<T> {
    pub const IN_VIEW: &str = "view";
}
impl<T: PostProcess> FromWorld for PostProcessNode<T> {
    fn from_world(world: &mut World) -> Self { Self { query: QueryState::new(world), marker: PhantomData } }
}
impl<T: PostProcess> Node for PostProcessNode<T> {
    fn input(&self) -> Vec<SlotInfo> { vec![SlotInfo::new(Self::IN_VIEW, SlotType::Entity)] }
    fn update(&mut self, world: &mut World) { self.query.update_archetypes(world); }
    fn run(
        &self, graph: &mut RenderGraphContext, render_context: &mut RenderContext, world: &World,
    ) -> Result<(), NodeRunError> {
        let view_entity = graph.get_input_entity(Self::IN_VIEW)?;
        let Ok((view_target, view_query)) = self.query.get_manual(world, view_entity) else { return Ok(()); };
        
        let post_process_pipeline = world.resource::<PostProcessPipeline<T>>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let Some(pipeline) = pipeline_cache.get_render_pipeline(post_process_pipeline.pipeline_id) else { return Ok(()); };

        let settings_uniforms = world.resource::<ComponentUniforms<T>>();
        let Some(settings_binding) = settings_uniforms.uniforms().binding() else { return Ok(()); };

        let post_process = view_target.post_process_write();
        let mut entries = vec![
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(post_process.source),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::Sampler(&post_process_pipeline.sampler),
            },
            BindGroupEntry {
                binding: 2,
                resource: settings_binding.clone(),
            },
        ];
        if let Some(bindings) = T::extract_bindings(view_query) {
            entries.extend_from_slice(&bindings);
        }
        let bind_group = render_context
            .render_device()
            .create_bind_group(&BindGroupDescriptor {
                label: Some(T::KEY),
                layout: &post_process_pipeline.layout,
                entries: &entries,
            });

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some(T::KEY),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: post_process.destination, resolve_target: None, ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
        });

        render_pass.set_render_pipeline(pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..3, 0..1);
        Ok(())
    }
}

#[derive(Resource)]
pub struct PostProcessPipeline<T: PostProcess> {
    layout: BindGroupLayout,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
    marker: PhantomData<T>,
}
impl<T: PostProcess> PostProcessPipeline<T> {

}
impl<T: PostProcess> FromWorld for PostProcessPipeline<T> {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let asset_server = world.resource::<AssetServer>();
        let shader = super::shared::load_shader(&asset_server, T::fragment_shader()).unwrap();
        let sampler = render_device.create_sampler(&SamplerDescriptor::default());

        let mut entries = vec![
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 2,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: bevy::render::render_resource::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ];
        T::bind_group_layout(&mut entries);
        let layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &entries,
        });
        let hdr = true;
        let pipeline_id = world
        .resource_mut::<PipelineCache>()
        .queue_render_pipeline(RenderPipelineDescriptor {
            label: None,
            layout: vec![layout.clone()],
            vertex: fullscreen_shader_vertex_state(),
            fragment: Some(FragmentState {
                shader, entry_point: "fragment".into(),
                shader_defs: vec![],
                targets: vec![Some(ColorTargetState {
                    format: if hdr { ViewTarget::TEXTURE_FORMAT_HDR } else { TextureFormat::bevy_default() },
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            push_constant_ranges: vec![],
        });
        
        Self { layout, sampler, pipeline_id, marker: PhantomData }
    }
}