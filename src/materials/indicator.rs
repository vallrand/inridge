use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup,ShaderRef};
use bevy::reflect::TypeUuid;
use bevy::sprite::Material2d;

#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "a3b744fe-8edd-425f-b068-03d9f7f8bf96"]
pub struct MatterIndicatorMaterial {
    #[uniform(0)]
    pub color: Color,
    #[uniform(0)]
    pub alpha_cutoff: f32,
    #[uniform(0)]
    pub fraction: f32,
    #[uniform(0)]
    pub fraction_width: f32,
    #[texture(1)]
    #[sampler(2)]
    pub texture: Handle<Image>,
}
impl Material2d for MatterIndicatorMaterial {
    fn fragment_shader() -> ShaderRef { "shaders/effect_indicator.wgsl".into() }
}

#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "6a0b3728-cf15-4d08-ae45-eda2045103ec"]
pub struct RadialIndicatorMaterial {
    #[uniform(0)]
    pub inner_color: Color,
    #[uniform(0)]
    pub outer_color: Color,
    #[uniform(0)]
    pub radius: Vec2,
    #[uniform(0)]
    pub sectors: u32,
    #[uniform(0)]
    pub fraction: f32,
    #[uniform(0)]
    pub padding: Vec2,
    #[uniform(0)]
    pub grid_resolution: Vec2,
}
impl Material2d for RadialIndicatorMaterial {
    fn fragment_shader() -> ShaderRef { "shaders/effect_radial.wgsl".into() }
}