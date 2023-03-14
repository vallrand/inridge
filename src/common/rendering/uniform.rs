use bevy::prelude::*;
use bevy::ecs::system::{SystemParamItem, lifetimeless::{Read, SRes}};
use bevy::ecs::query::ROQueryItem;
use bevy::render::render_phase::{PhaseItem, RenderCommand, RenderCommandResult, TrackedRenderPass};
use bevy::render::{renderer::RenderDevice, RenderApp, RenderSet,};
use std::marker::PhantomData;
use bevy::render::render_resource::encase::private::WriteInto;
use bevy::render::render_resource::{
    BindGroupDescriptor, BindGroup, BindGroupLayout, ShaderType
};
use bevy::render::render_resource::BindGroupEntry;
use bevy::render::extract_component::{
    ComponentUniforms, DynamicUniformIndex, UniformComponentPlugin, ExtractComponentPlugin, ExtractComponent
};

pub trait ShaderUniform: Component + ShaderType + Clone + 'static {

}
impl<T: Component + ShaderType + Clone + 'static> ShaderUniform for T {

}
pub struct ShaderUniformPlugin<U: ShaderUniform>(PhantomData<fn() -> U>);
impl<U: ShaderUniform + ExtractComponent + WriteInto> Default for ShaderUniformPlugin<U> { fn default() -> Self { Self(PhantomData) } }
impl<U: ShaderUniform + ExtractComponent + WriteInto> Plugin for ShaderUniformPlugin<U> {
    fn build(&self, app: &mut App) {
        app.add_plugin(ExtractComponentPlugin::<U>::extract_visible());
        app.add_plugin(UniformComponentPlugin::<U>::default());
        app.sub_app_mut(RenderApp)
        .init_resource::<UniformBindGroupLayout<U>>()
        .add_system(queue_bind_group::<U>.in_set(RenderSet::Queue));
    }
}

#[derive(Resource)]
pub struct UniformBindGroupLayout<U: ShaderUniform> {
    marker: PhantomData<U>,
    pub bind_group_layout: BindGroupLayout,
}
impl<U: ShaderUniform> FromWorld for UniformBindGroupLayout<U> {
    fn from_world(render_world: &mut World) -> Self {
        use bevy::render::render_resource::*;
        let render_device = render_world.resource::<RenderDevice>();
        Self {
            marker: PhantomData,
            bind_group_layout: render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: None,
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: Some(U::min_size()),
                    },
                    count: None,
                }]
            })
        }
    }
}

#[derive(Resource)] pub struct UniformBindGroup<U: ShaderUniform> {
    pub value: BindGroup,
    marker: PhantomData<U>,
}

pub fn queue_bind_group<U: ShaderUniform + WriteInto>(
    mut commands: Commands,
    uniform_layout: Res<UniformBindGroupLayout<U>>,
    render_device: Res<RenderDevice>,
    uniforms: Res<ComponentUniforms<U>>,
){
    if let Some(binding) = uniforms.uniforms().binding() {
        commands.insert_resource(UniformBindGroup::<U> {
            marker: PhantomData,
            value: render_device.create_bind_group(&BindGroupDescriptor {
                label: None,
                entries: &[BindGroupEntry {
                    binding: 0, resource: binding
                }],
                layout: &uniform_layout.bind_group_layout,
            })
        });
    }
}

pub struct SetUniformBindGroup<U: ShaderUniform, const I: usize>(PhantomData<U>);
impl<P: PhaseItem, U: ShaderUniform, const I: usize> RenderCommand<P> for SetUniformBindGroup<U, I> {
    type Param = SRes<UniformBindGroup<U>>;
    type ViewWorldQuery = ();
    type ItemWorldQuery = Read<DynamicUniformIndex<U>>;
    #[inline] fn render<'w>(
        _item: &P, _view: ROQueryItem<'w, Self::ViewWorldQuery>,
        dynamic_uniform_indices: ROQueryItem<'w, Self::ItemWorldQuery>,
        bind_group: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        pass.set_bind_group(I, &bind_group.into_inner().value, &[dynamic_uniform_indices.index()]);
        RenderCommandResult::Success
    }
}
