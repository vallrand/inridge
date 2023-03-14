use bevy::prelude::*;
use bevy::render::{RenderApp, RenderSet};
use bevy::render::render_resource::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, BlendState, BufferBindingType,
    CachedRenderPipelineId, ColorTargetState, ColorWrites, CompareFunction, DepthBiasState,
    DepthStencilState, FragmentState, MultisampleState, PipelineCache, PrimitiveState,
    RenderPipelineDescriptor, Shader, ShaderStages, ShaderType,
    SpecializedRenderPipeline, SpecializedRenderPipelines, StencilFaceState, StencilState,
    TextureFormat, VertexState, ShaderRef,
};
use bevy::render::renderer::RenderDevice;
use bevy::render::texture::BevyDefault;
use bevy::render::view::{ExtractedView, Msaa, ViewTarget, ViewUniform, ViewUniforms};
use bevy::render::render_resource::encase::private::WriteInto;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin, UniformComponentPlugin, ComponentUniforms};
use bevy::pbr::MeshPipelineKey;
use bevy::render::globals::{GlobalsUniform, GlobalsBuffer};
use std::marker::PhantomData;
use super::shared::load_shader;

pub trait SkyboxMaterial: Component + ExtractComponent + ShaderType + WriteInto + Clone + Sync + Send + 'static {
    fn fragment_shader() -> ShaderRef { ShaderRef::Default }
}

pub struct SkyboxPlugin<M: SkyboxMaterial>(PhantomData<M>);
impl<M: SkyboxMaterial> Default for SkyboxPlugin<M> { fn default() -> Self { Self(PhantomData) } }
impl<M: SkyboxMaterial> Plugin for SkyboxPlugin<M>
where M::Out: Component {
    fn build(&self, app: &mut App) {
        app.add_plugin(ExtractComponentPlugin::<M>::default());
        app.add_plugin(UniformComponentPlugin::<M>::default());
        app.sub_app_mut(RenderApp)
            .init_resource::<SkyboxPipeline<M>>()
            .init_resource::<SpecializedRenderPipelines<SkyboxPipeline<M>>>()
            .add_system(queue_skybox::<M>.in_set(RenderSet::Queue));
        
    }
}

#[derive(Resource)]
struct SkyboxPipeline<M: SkyboxMaterial> {
    shader: Handle<Shader>,
    bind_group_layout: BindGroupLayout,
    marker: PhantomData<M>,
}
impl<M: SkyboxMaterial> FromWorld for SkyboxPipeline<M> {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let asset_server = world.resource::<AssetServer>();
        Self {
            bind_group_layout: render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("skybox_bind_group_layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::VERTEX_FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: true,
                            min_binding_size: Some(ViewUniform::min_size()),
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::VERTEX_FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: Some(GlobalsUniform::min_size()),
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: Some(M::min_size()),
                        },
                        count: None,
                    },
                ],
            }),
            shader: load_shader(&asset_server, M::fragment_shader()).unwrap(),
            marker: PhantomData,
        }
    }
}

impl<M: SkyboxMaterial> SpecializedRenderPipeline for SkyboxPipeline<M> {
    type Key = MeshPipelineKey;
    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        RenderPipelineDescriptor {
            label: Some("skybox_pipeline".into()),
            layout: vec![self.bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            vertex: VertexState {
                shader: self.shader.clone(), entry_point: "vertex".into(),
                shader_defs: Vec::new(), buffers: Vec::new(),
            },
            primitive: PrimitiveState::default(),
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: CompareFunction::GreaterEqual,
                stencil: StencilState {
                    front: StencilFaceState::IGNORE,
                    back: StencilFaceState::IGNORE,
                    read_mask: 0, write_mask: 0,
                },
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState {
                count: key.msaa_samples(), mask: !0, alpha_to_coverage_enabled: false,
            },
            fragment: Some(FragmentState {
                shader: self.shader.clone(), entry_point: "fragment".into(),
                shader_defs: Vec::new(),
                targets: vec![Some(ColorTargetState {
                    format: if key.contains(MeshPipelineKey::HDR) {
                        ViewTarget::TEXTURE_FORMAT_HDR
                    } else {
                        TextureFormat::bevy_default()
                    },
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
        }
    }
}

fn queue_skybox<M: SkyboxMaterial>(
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedRenderPipelines<SkyboxPipeline<M>>>,
    pipeline: Res<SkyboxPipeline<M>>,
    view_uniforms: Res<ViewUniforms>,
    globals_buffer: Res<GlobalsBuffer>,
    uniforms: Res<ComponentUniforms<M>>,
    render_device: Res<RenderDevice>,
    msaa: Res<Msaa>,
    mut views: Query<(&ExtractedView, &M::Out, &mut FullscreenPhase)>,
) where M::Out: Component {
    for (view, _skybox, mut phase) in &mut views {
        let key = MeshPipelineKey::from_hdr(view.hdr) | MeshPipelineKey::from_msaa_samples(msaa.samples());
        let pipeline_id = pipelines.specialize(
            &pipeline_cache, &pipeline, key
        );
        let (
            Some(globals_binding),
            Some(view_binding),
            Some(component_binding),
        ) = (
            globals_buffer.buffer.binding(),
            view_uniforms.uniforms.binding(),
            uniforms.uniforms().binding(),
        ) else { continue };

        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: None, layout: &pipeline.bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0, resource: view_binding,
                },
                BindGroupEntry {
                    binding: 1, resource: globals_binding,
                },
                BindGroupEntry {
                    binding: 2, resource: component_binding,
                },
            ],
        });

        phase.push((pipeline_id, bind_group));
    }
}

use bevy::render::render_graph::{Node, NodeRunError, RenderGraph, RenderGraphContext, SlotInfo, SlotType};
use bevy::render::render_resource::{Operations, LoadOp, RenderPassDepthStencilAttachment, RenderPassDescriptor};
use bevy::render::camera::ExtractedCamera;
use bevy::render::view::{ViewDepthTexture, ViewUniformOffset};
use bevy::core_pipeline::core_3d;
use bevy::ecs::system::lifetimeless::Read;

#[derive(Component, Deref, DerefMut, Default)]
pub struct FullscreenPhase(pub Vec<(CachedRenderPipelineId, BindGroup)>);

pub fn prepare_fullscreen_bindings(mut commands: Commands, views: Query<Entity, With<ExtractedView>>){
    for entity in &views {
        commands.entity(entity).insert(FullscreenPhase::default());
    }
}
pub struct FullscreenPhasePlugin; impl Plugin for FullscreenPhasePlugin {
    fn build(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_system(prepare_fullscreen_bindings.in_set(RenderSet::Prepare));
        FullscreenNode::attach(render_app);
    }
}
pub struct FullscreenNode {
    query: QueryState<(
        Read<ViewTarget>,
        Read<ViewDepthTexture>,
        Read<ExtractedCamera>,
        Read<ViewUniformOffset>,
        Read<FullscreenPhase>,
    ), With<ExtractedView>>,
}
impl FullscreenNode {
    pub const IN_VIEW: &str = "view";
    pub const KEY: &str = "fullscreen";
    pub fn attach(render_app: &mut App){
        let node = Self::from_world(&mut render_app.world);
        let mut graph = render_app.world.resource_mut::<RenderGraph>();
        let subgraph = graph.get_sub_graph_mut(core_3d::graph::NAME).unwrap();
        subgraph.add_node(Self::KEY, node);
        subgraph.add_slot_edge(
            subgraph.input_node().id, core_3d::graph::input::VIEW_ENTITY,
            Self::KEY, Self::IN_VIEW,
        );
        subgraph.add_node_edge(core_3d::graph::node::PREPASS, Self::KEY);
        subgraph.add_node_edge(Self::KEY, core_3d::graph::node::MAIN_PASS);
    }
}
impl FromWorld for FullscreenNode {
    fn from_world(world: &mut World) -> Self { Self { query: QueryState::new(world) } }
}
impl Node for FullscreenNode {
    fn input(&self) -> Vec<SlotInfo> { vec![SlotInfo::new(Self::IN_VIEW, SlotType::Entity)] }
    fn update(&mut self, world: &mut World) { self.query.update_archetypes(world); }
    fn run(
        &self, graph: &mut RenderGraphContext, render_context: &mut bevy::render::renderer::RenderContext, world: &World,
    ) -> Result<(), NodeRunError> {
        let view_entity = graph.get_input_entity(Self::IN_VIEW)?;
        let Ok((
            target, depth, camera,
            view_uniform_offset, bindings
        )) = self.query.get_manual(world, view_entity) else { return Ok(()); };

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(target.get_color_attachment(Operations {
                load: LoadOp::Load, store: true,
            }))],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: &depth.view,
                depth_ops: Some(Operations {
                    load: LoadOp::Load,
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        if let Some(viewport) = camera.viewport.as_ref() {
            render_pass.set_camera_viewport(viewport);
        }
      
        let pipeline_cache = world.resource::<PipelineCache>();
        for (pipeline_id, bind_group) in bindings.iter() {
            let Some(pipeline) = pipeline_cache.get_render_pipeline(*pipeline_id) else { continue };
            render_pass.set_render_pipeline(pipeline);
            render_pass.set_bind_group(0, bind_group, &[view_uniform_offset.offset]);
            render_pass.draw(0..3, 0..1);
        }

        Ok(())
    }
}