use std::cmp::Reverse;
use std::marker::PhantomData;
use bevy::prelude::*;
use bevy::utils::{HashMap, FloatOrd};
use bevy::render::camera::ExtractedCamera;
use bevy::render::render_resource::{Extent3d, TextureDescriptor, TextureDimension, TextureUsages};
use bevy::render::render_phase::{
    RenderPhase, CachedRenderPipelinePhaseItem, DrawFunctionId, PhaseItem, DrawFunctions, sort_phase_system
};
use bevy::core_pipeline::core_3d;
use bevy::render::render_graph::{Node, NodeRunError, RenderGraph, RenderGraphContext, SlotInfo, SlotType};
use bevy::render::render_resource::{
    CachedRenderPipelineId, Operations, RenderPassColorAttachment, RenderPassDescriptor, TextureFormat,
    RenderPassDepthStencilAttachment, LoadOp
};
use bevy::render::renderer::{RenderContext, RenderDevice};
use bevy::render::texture::{CachedTexture, TextureCache};
use bevy::render::view::{ExtractedView, ViewDepthTexture};
use bevy::render::{RenderApp, RenderSet, Extract};

pub trait RenderTexturePass: Component + Sync + Send +'static {
    const TEXTURE_FORMAT: TextureFormat = TextureFormat::Rgba8Unorm;
    const INPUT_NODE: &'static str = core_3d::graph::node::MAIN_PASS;
    const OUTPUT_NODE: &'static str = core_3d::graph::node::TONEMAPPING;
    const GRAPH_KEY: &'static str = core_3d::graph::NAME;
    const KEY: &'static str;
}

pub struct RenderTexturePassPlugin<T>(PhantomData<T>);
impl<T: RenderTexturePass> Default for RenderTexturePassPlugin<T> { fn default() -> Self { Self(PhantomData) } }
impl<T: RenderTexturePass> Plugin for RenderTexturePassPlugin<T> {
    fn build(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<DrawFunctions<RenderTexturePhase<T>>>();
        render_app.add_system(sort_phase_system::<RenderTexturePhase<T>>.in_set(RenderSet::PhaseSort));
        render_app.add_system(extract_camera_view::<T>.in_schedule(ExtractSchedule));
        render_app.add_system(prepare_render_texture::<T>.in_set(RenderSet::Prepare)
            .after(bevy::render::view::prepare_windows));

        let node = RenderPhaseNode::<T>::from_world(&mut render_app.world);
        let mut graph = render_app.world.resource_mut::<RenderGraph>();
        let subgraph = graph.get_sub_graph_mut(T::GRAPH_KEY).unwrap();
        subgraph.add_node(T::KEY, node);
        subgraph.add_slot_edge(
            subgraph.input_node().id, core_3d::graph::input::VIEW_ENTITY,
            T::KEY, RenderPhaseNode::<T>::IN_VIEW,
        );
        subgraph.add_node_edge(T::INPUT_NODE, T::KEY);
        subgraph.add_node_edge(T::KEY, T::OUTPUT_NODE);
    }
}

#[derive(Component)]
pub struct ViewRenderTexture<T: RenderTexturePass> {
    pub render_texture: CachedTexture,
    pub size: Extent3d,
    marker: PhantomData<T>,
}

pub struct RenderTexturePhase<T: RenderTexturePass> {
    pub distance: f32,
    pub entity: Entity,
    pub pipeline_id: CachedRenderPipelineId,
    pub draw_function: DrawFunctionId,
    marker: PhantomData<T>,
}
impl<T: RenderTexturePass> CachedRenderPipelinePhaseItem for RenderTexturePhase<T> {
    #[inline] fn cached_pipeline(&self) -> CachedRenderPipelineId { self.pipeline_id }
}
impl<T: RenderTexturePass> PhaseItem for RenderTexturePhase<T> {
    type SortKey = Reverse<FloatOrd>;
    #[inline] fn entity(&self) -> Entity { self.entity }
    #[inline] fn sort_key(&self) -> Self::SortKey { Reverse(FloatOrd(self.distance)) }
    #[inline] fn draw_function(&self) -> DrawFunctionId { self.draw_function }
}
use crate::common::rendering::CreateItemBuilder;
impl<T: RenderTexturePass> CreateItemBuilder for RenderTexturePhase<T> {
    fn new(entity: Entity, draw_function: DrawFunctionId, pipeline: CachedRenderPipelineId, distance: f32) -> Self {
        Self { entity, draw_function, pipeline_id: pipeline, distance, marker: PhantomData }
    }
}

pub struct RenderPhaseNode<T: RenderTexturePass> {
    view_query: QueryState<(
        &'static ExtractedCamera,
        &'static RenderPhase<RenderTexturePhase<T>>,
        &'static ViewDepthTexture,
        &'static ViewRenderTexture<T>,
    ), With<ExtractedView>>
}
impl<T: RenderTexturePass> RenderPhaseNode<T> {
    pub const IN_VIEW: &'static str = "view";
}
impl<T: RenderTexturePass> FromWorld for RenderPhaseNode<T> {
    fn from_world(world: &mut World) -> Self { Self { view_query: QueryState::new(world) } }
}
impl<T: RenderTexturePass> Node for RenderPhaseNode<T> {
    fn input(&self) -> Vec<SlotInfo> { vec![SlotInfo::new(Self::IN_VIEW, SlotType::Entity)] }
    fn update(&mut self, world: &mut World) { self.view_query.update_archetypes(world); }
    fn run(
        &self, graph: &mut RenderGraphContext, render_context: &mut RenderContext, world: &World,
    ) -> Result<(), NodeRunError> {
        let view_entity = graph.get_input_entity(Self::IN_VIEW)?;
        let Ok((
            camera, render_phase, view_depth_texture, view_render_texture
        )) = self.view_query.get_manual(world, view_entity) else { return Ok(()) };

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view_render_texture.render_texture.default_view,
                resolve_target: None,
                ops: Operations { load: LoadOp::Clear(Color::NONE.into()), store: true },
            })],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: &view_depth_texture.view,
                depth_ops: Some(Operations { load: LoadOp::Load, store: true }),
                stencil_ops: None,
            })
        });
        if let Some(viewport) = &camera.viewport {
            render_pass.set_camera_viewport(viewport);
        }
        render_phase.render(&mut render_pass, world, view_entity);

        Ok(())    
    }
}

pub fn extract_camera_view<T: RenderTexturePass>(
    mut commands: Commands,
    cameras: Extract<Query<(Entity, &Camera), With<T>>>,
){
    for (entity, camera) in &cameras {
        if !camera.is_active { continue; }
        let mut entity = commands.get_or_spawn(entity);
        entity.insert(RenderPhase::<RenderTexturePhase<T>>::default());
    }
}
pub fn prepare_render_texture<T: RenderTexturePass>(
    mut commands: Commands,
    mut texture_cache: ResMut<TextureCache>,
    msaa: Res<Msaa>,
    render_device: Res<RenderDevice>,
    views: Query<(Entity, &ExtractedCamera), With<RenderPhase<RenderTexturePhase<T>>>>,
){
    let mut textures = HashMap::default();
    for (entity, camera) in &views {
        let Some(physical_target_size) = camera.physical_target_size else { continue };
        let size = Extent3d {
            depth_or_array_layers: 1, width: physical_target_size.x, height: physical_target_size.y,
        };
        let cached_texture = textures.entry(camera.target.clone())
        .or_insert_with(||
            texture_cache.get(&render_device, TextureDescriptor {
                label: None, size,
                mip_level_count: 1,
                sample_count: msaa.samples(),
                dimension: TextureDimension::D2,
                format: T::TEXTURE_FORMAT,
                usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                view_formats: &[],
            })
        ).clone();

        commands.entity(entity).insert(ViewRenderTexture::<T>{
            render_texture: cached_texture, size, marker: PhantomData
        });
    }
}