use bevy::prelude::*;
use bevy::render::extract_component::{ExtractComponent,ExtractComponentPlugin};
use bevy::render::{RenderApp, RenderSet};
use bevy::pbr::MeshUniform;

#[derive(Component, ExtractComponent, Clone, Copy, Reflect, Default)]
#[reflect(Component, Default)]
pub struct ColorUniform {
    pub color: Color
}
impl From<Color> for ColorUniform { #[inline] fn from(color: Color) -> Self { Self { color } } }
impl From<ColorUniform> for Color { #[inline] fn from(value: ColorUniform) -> Self { value.color } }

pub struct ColorUniformPlugin; impl Plugin for ColorUniformPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ColorUniform>();
        app.add_plugin(ExtractComponentPlugin::<ColorUniform>::extract_visible());
        app.sub_app_mut(RenderApp)
            .add_system(prepare_mesh_uniform_components
                .after(RenderSet::ExtractCommands)
                .before(RenderSet::Prepare));
    }
}

fn prepare_mesh_uniform_components(
    mut components: Query<(&ColorUniform, &mut MeshUniform), Without<Handle<StandardMaterial>>>
){
    for (color_uniform, mut mesh_uniform) in components.iter_mut() {
        let vec = color_uniform.color.as_rgba_f32();
        mesh_uniform.transform.x_axis.w = vec[0];
        mesh_uniform.transform.y_axis.w = vec[1];
        mesh_uniform.transform.z_axis.w = vec[2];
        mesh_uniform.transform.w_axis.w = vec[3];
    }
}

use crate::common::animation::{AnimatedProperty,ease::lerp};
impl AnimatedProperty for ColorUniform {
    type Component = ColorUniform;
    const DEFAULT: Self = Self{color:Color::NONE};
    fn lerp(&self, rhs: &Self, fraction: f32) -> Self {
        let prev = self.color.as_linear_rgba_f32();
        let next = rhs.color.as_linear_rgba_f32();
        Self{ color: Color::rgba_linear(
            lerp(prev[0], next[0], fraction),
            lerp(prev[1], next[1], fraction),
            lerp(prev[2], next[2], fraction),
            lerp(prev[3], next[3], fraction),
        ) }
    }
    fn blend(&self, weight: f32, target: &mut Self::Component) {
        target.color = Self::lerp(&target.color.into(), self, weight).color;
    }
}