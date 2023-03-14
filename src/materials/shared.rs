use bevy::render::render_resource::{RenderPipelineDescriptor,Face};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Billboard {
    Axial,
    Spherical,
    AxialScreen,
    SphericalScreen,
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct EffectMaterialKey {
    pub color_mask: bool,
    pub fresnel_mask: bool,
    pub depth_mask: bool,
    pub alpha_mask: bool,
    pub billboard: Option<Billboard>,
    pub diffuse_map: bool,
    pub displacement_map: bool,
    pub depth_bias: i32,
    pub cull_mode: Option<Face>
}

impl Default for EffectMaterialKey {
    fn default() -> Self { Self {
        color_mask: false,
        fresnel_mask: false,
        depth_mask: false,
        alpha_mask: false,
        billboard: None,
        diffuse_map: false,
        displacement_map: false,
        depth_bias: 0,
        cull_mode: Some(Face::Back)
    } }
}

impl EffectMaterialKey {
    pub fn apply(&self, descriptor: &mut RenderPipelineDescriptor){
        let mut shader_defs = vec![];
        if self.diffuse_map { shader_defs.push("DIFFUSE_MAP".into()); }
        if self.displacement_map { shader_defs.push("DISPLACEMENT_MAP".into()); }
        if self.fresnel_mask { shader_defs.push("FRESNEL_MASK".into()); }
        if self.fresnel_mask { shader_defs.push("DEPTH_MASK".into()); }
        if self.alpha_mask { shader_defs.push("ALPHA_MASK".into()); }
        if let Some(billboard) = &self.billboard {
            shader_defs.push("BILLBOARD".into());
            if Billboard::AxialScreen.eq(billboard) || Billboard::SphericalScreen.eq(billboard) {
                shader_defs.push("SCREEN_ALIGNED".into());
            }
        }
        if self.billboard.is_some() { shader_defs.push("BILLBOARD".into()); }
        if self.color_mask { shader_defs.push("VERTEX_COLOR_MASK".into()); }

        descriptor.vertex.shader_defs.extend_from_slice(&shader_defs);
        if let Some(fragment) = descriptor.fragment.as_mut() {
            fragment.shader_defs.extend_from_slice(&shader_defs);
        }
        
        descriptor.primitive.cull_mode = self.cull_mode;
    }
}