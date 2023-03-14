
use bevy::prelude::*;
use bevy::render::render_resource::{ShaderType, ShaderRef};
use bevy::render::extract_component::ExtractComponent;
use crate::extensions::SkyboxMaterial;

#[derive(Component, ShaderType, ExtractComponent, Clone, Copy, Default)]
pub struct SkyboxNebula {
    pub color: Color,
}

impl SkyboxMaterial for SkyboxNebula {
    fn fragment_shader() -> ShaderRef { "shaders/skybox_nebula.wgsl".into() }
}