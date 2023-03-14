use bevy::prelude::*;
use bevy::pbr::{MeshPipelineKey,MeshUniform,};
use bevy::asset::{Assets, Handle, HandleId};
use bevy::ecs::system::{
    lifetimeless::{Read, SRes},
    SystemParamItem,
};
use bevy::render::render_resource::{
    CachedRenderPipelineId, Buffer, BufferInitDescriptor, BufferUsages,
    VertexBufferLayout, VertexStepMode, VertexAttribute, VertexFormat,
};
use bevy::render::{
    mesh::{Mesh, GpuBufferInfo},
    render_asset::RenderAssets,
    render_phase::{
        DrawFunctions, PhaseItem, RenderCommand, RenderCommandResult,
        RenderPhase, TrackedRenderPass,
    },
    render_resource::{PipelineCache, SpecializedMeshPipelines},
    renderer::RenderDevice,
    view::{ExtractedView, Msaa, VisibleEntities},
};
use std::hash::Hash;
use bevy::utils::HashMap;
use bevy::ecs::query::ROQueryItem;

use super::extract::PreparedRenderAssets;
use super::material::*;

#[inline] pub fn try_cast_slice<A: Copy + Sized, B: Copy + Sized>(a: &[A]) -> Result<&[B], &'static str> {
    use core::mem::{align_of,size_of,size_of_val};
    use core::slice::from_raw_parts;
    if align_of::<B>() > align_of::<A>() && (a.as_ptr() as usize) % align_of::<B>() != 0 {
        Err("TargetAlignmentGreaterAndInputNotAligned")
    } else if size_of::<B>() == size_of::<A>() {
        Ok(unsafe { from_raw_parts(a.as_ptr() as *const B, a.len()) })
    } else if size_of::<A>() == 0 || size_of::<B>() == 0 {
        Err("SizeMismatch")
    } else if size_of_val(a) % size_of::<B>() == 0 {
        let new_len = size_of_val(a) / size_of::<B>();
        Ok(unsafe { from_raw_parts(a.as_ptr() as *const B, new_len) })
    } else {
        Err("OutputSliceWouldHaveSlop")
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct InstanceVertexAttributes {
    row_x: [f32; 4],
    row_y: [f32; 4],
    row_z: [f32; 4],
    row_w: [f32; 4],
}

impl InstanceVertexAttributes {
    pub fn from_mesh_uniform(uniform: &MeshUniform) -> Self {
        let Mat4 { x_axis, y_axis, z_axis, w_axis } = &uniform.transform;
        Self {
            row_x: [x_axis.x, y_axis.x, z_axis.x, w_axis.x],
            row_y: [x_axis.y, y_axis.y, z_axis.y, w_axis.y],
            row_z: [x_axis.z, y_axis.z, z_axis.z, w_axis.z],
            row_w: [0.0,0.0,0.0,1.0]
        }
    }
    pub fn layout() -> VertexBufferLayout { VertexBufferLayout {
        array_stride: std::mem::size_of::<InstanceVertexAttributes>() as u64,
        step_mode: VertexStepMode::Instance,
        attributes: (0..4 as u32).map(|i|VertexAttribute {
            format: VertexFormat::Float32x4,
            offset: i as u64 * VertexFormat::Float32x4.size(),
            shader_location: 7 + i,
        }).collect::<Vec<VertexAttribute>>()
    } }
}

pub fn queue_instanced_meshes<M: ShaderMaterial>(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    draw_functions: Res<DrawFunctions<M::Phase>>,
    specialize_pipeline: Res<ShaderMaterialPipeline<M>>,
    mut pipelines: ResMut<SpecializedMeshPipelines<ShaderMaterialPipeline<M>>>,
    pipeline_cache: Res<PipelineCache>,
    mut list_pool: Local<Vec<Vec<InstanceVertexAttributes>>>,

    msaa: Res<Msaa>,
    render_meshes: Res<RenderAssets<Mesh>>,
    render_materials: Res<PreparedRenderAssets<PreparedMaterial<M>>>,
    query_instance: Query<(&Handle<M>, &Handle<Mesh>, &MeshUniform), M::Filter>,
    mut views: Query<(&ExtractedView, &VisibleEntities, &mut RenderPhase<M::Phase>)>,
) where M::Data: PartialEq + Eq + Hash + Clone {
    let draw_function = draw_functions.read().id::<M::DrawCommand>();

    for (view, visible_entities, mut render_phase) in &mut views {
        let view_key = MeshPipelineKey::from_msaa_samples(msaa.samples()) | MeshPipelineKey::from_hdr(view.hdr);
        let mut batches: HashMap<
            (CachedRenderPipelineId, HandleId, HandleId),
            (Vec<InstanceVertexAttributes>, Handle<Mesh>, Handle<M>)
        > = HashMap::new();

        let rangefinder = view.rangefinder3d();
        for visible_entity in &visible_entities.entities {
            let Ok((material_handle, mesh_handle, mesh_uniform)) =
            query_instance.get(*visible_entity) else { continue };
            
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

            let entry = batches
            .entry((pipeline_id, mesh_handle.id(), material_handle.id()))
            .or_insert_with(||(
                list_pool.pop().unwrap_or_default(),
                mesh_handle.clone_weak(),
                material_handle.clone_weak()
            ));
            
            let _distance = rangefinder.distance(&mesh_uniform.transform) + material.depth_bias;
            entry.0.push(InstanceVertexAttributes::from_mesh_uniform(mesh_uniform));
        }
        for (key, (mut instances, mesh_handle, material_handle)) in batches.into_iter() {
            let Some(buffer) = try_cast_slice::<InstanceVertexAttributes, u8>(instances.as_slice()).ok()
            .map(|contents|render_device.create_buffer_with_data(&BufferInitDescriptor {
                label: None, usage: BufferUsages::VERTEX | BufferUsages::COPY_DST, contents
            })) else { continue };

            let batch_entity = commands.spawn((
                InstanceBuffer { buffer, length: instances.len() },
                mesh_handle,
                material_handle,
            )).id();

            instances.clear();
            list_pool.push(instances);
            render_phase.add(M::Phase::new(batch_entity, draw_function, key.0, 0.0));
        }
    }
}

#[derive(Component)]
pub struct InstanceBuffer {
    buffer: Buffer,
    length: usize,
}

pub struct DrawMeshInstanced; impl<P: PhaseItem> RenderCommand<P> for DrawMeshInstanced {
    type Param = SRes<RenderAssets<Mesh>>;
    type ViewWorldQuery = ();
    type ItemWorldQuery = (Read<Handle<Mesh>>, Read<InstanceBuffer>);
    #[inline] fn render<'w>(
        _item: &P, _view: ROQueryItem<'w, Self::ViewWorldQuery>,
        (mesh_handle, instance_buffer): ROQueryItem<'w, Self::ItemWorldQuery>,
        param: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(gpu_mesh) = param.into_inner().get(mesh_handle) else { return RenderCommandResult::Failure };
        pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
        pass.set_vertex_buffer(1, instance_buffer.buffer.slice(..));
        match &gpu_mesh.buffer_info {
            GpuBufferInfo::Indexed { buffer, index_format, count } => {
                pass.set_index_buffer(buffer.slice(..), 0, *index_format);
                pass.draw_indexed(0..*count, 0, 0..instance_buffer.length as u32);
            },
            GpuBufferInfo::NonIndexed { vertex_count } => {
                pass.draw(0..*vertex_count, 0..instance_buffer.length as u32);
            }
        }
        RenderCommandResult::Success
    }
}

pub fn bake_skeletal_animation(
    animations: Res<Assets<AnimationClip>>,
    handles: &[Handle<AnimationClip>],
){
    for handle in handles.iter() {
        let animation = animations.get(handle).unwrap();
        let duration = animation.duration();
        let curves = animation.curves();
        //TODO
    }
}