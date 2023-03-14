use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef, Face};
use bevy::reflect::TypeUuid;

#[derive(AsBindGroup, TypeUuid, Clone, Default, Debug)]
#[uuid = "4a767564-692a-49db-8175-f7e2f7a5cfae"]
#[bind_group_data(EffectMaterialKey)]
pub struct ModelEffectLayeredMaterial {
    #[texture(0, dimension = "2d_array")]
    #[sampler(1)]
    pub albedo: Handle<Image>,
    #[texture(2, dimension = "2d_array")]
    #[sampler(3)]
    pub normal: Handle<Image>,
    #[texture(4, dimension = "2d_array")]
    #[sampler(5)]
    pub rma: Handle<Image>,

    pub dissolve: bool,
    pub damage: bool,

    #[uniform(6)]
    pub uv_transform: Vec4,
    #[uniform(6)]
    pub emission: f32,
    #[uniform(6)]
    pub noise_domain: Vec3,
    #[uniform(6)]
    pub alpha_threshold: f32,
    #[uniform(6)]
    pub color_shift: Color,
    #[uniform(6)]
    pub scanline_color: Color,
    #[uniform(6)]
    pub scanline_width: Vec4,
    #[uniform(6)]
    pub dissolve_color: Color,
    #[uniform(6)]
    pub dissolve_plane: Vec4,
    #[uniform(6)]
    pub dissolve_offset: Vec2,
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct EffectMaterialKey {
    pub dissolve: bool,
    pub damage: bool,
}
impl From<&ModelEffectLayeredMaterial> for EffectMaterialKey {
    fn from(value: &ModelEffectLayeredMaterial) -> Self { Self {
        dissolve: value.dissolve,
        damage: value.damage,
    } }
}

impl Material for ModelEffectLayeredMaterial {
    fn fragment_shader() -> ShaderRef { "shaders/model_effect.wgsl".into() }
    fn vertex_shader() -> ShaderRef { "shaders/model_effect.wgsl".into() }
    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline<Self>,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        _layout: &bevy::render::mesh::MeshVertexBufferLayout,
        key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        if let Some(fragment) = descriptor.fragment.as_mut() {
            fragment.shader_defs.push("FRAGMENT".into());
            if key.bind_group_data.dissolve {
                descriptor.primitive.cull_mode = None;
                descriptor.vertex.shader_defs.push("DISSOLVE".into());
                fragment.shader_defs.push("DISSOLVE".into());
            } else {
                descriptor.primitive.cull_mode = Some(Face::Back);
            }
            if key.bind_group_data.damage {
                descriptor.vertex.shader_defs.push("DAMAGE".into());
                fragment.shader_defs.push("DAMAGE".into());
            }
        }
        Ok(())
    }
}