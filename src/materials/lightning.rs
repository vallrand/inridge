use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::render::mesh::PrimitiveTopology;
use bevy::render::mesh::{Indices, Mesh};
use crate::common::geometry::sphere::fibonacci_lattice;
use crate::common::noise::MurMurHash;
use super::shared::{EffectMaterialKey, Billboard};

#[derive(Clone, Default)]
pub struct HairBall {
    pub seed: u32,
    pub radius: f32,
    pub width: f32,
    pub quantity: usize,
    pub hemisphere: bool,
}
impl From<HairBall> for Mesh {
    fn from(value: HairBall) -> Self {
        let mut vertex_positions: Vec<[f32; 3]> = Vec::with_capacity(value.quantity * 4);
        let mut vertex_normals: Vec<[f32; 3]> = Vec::with_capacity(value.quantity * 4);
        let mut vertex_tangents: Vec<[f32; 4]> = Vec::with_capacity(value.quantity * 4);
        let mut vertex_uvs: Vec<[f32; 2]> = Vec::with_capacity(value.quantity * 4);
        let mut vertex_colors: Vec<[f32; 4]> = Vec::with_capacity(value.quantity * 4);
        let mut indices: Vec<u32> = Vec::with_capacity(value.quantity * 6);
        let mut hash = MurMurHash::from_seed(value.seed as u64);

        for binormal in fibonacci_lattice(value.quantity).into_iter()
        .take(if value.hemisphere { value.quantity / 2 }else{ value.quantity }) {
            let binormal = Vec3::from(binormal);
            let tangent = Vec3::cross(binormal, binormal.any_orthonormal_vector());
            let normal = Vec3::cross(binormal, tangent);

            vertex_positions.extend_from_slice(&[
                (Vec3::ZERO + tangent * value.width).to_array(),
                (Vec3::ZERO - tangent * value.width).to_array(),
                (binormal * value.radius - tangent * value.width).to_array(),
                (binormal * value.radius + tangent * value.width).to_array(),
            ]);
            vertex_normals.extend_from_slice(&[normal.to_array(); 4]);
            vertex_tangents.extend_from_slice(&[tangent.extend(value.width).to_array(); 4]);
            vertex_uvs.extend_from_slice(&[[0.0,0.0],[1.0,0.0],[1.0,1.0],[0.0,1.0]]);
            vertex_colors.extend_from_slice(&[[hash.next_f32(), hash.next_f32(), 0.0, 1.0]; 4]);
        }
        for i in 0..value.quantity as u32 {
            indices.extend_from_slice(&[
                i * 4 + 0, i * 4 + 1, i * 4 + 2,
                i * 4 + 0, i * 4 + 2, i * 4 + 3,
            ]);
        }

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(Indices::U32(indices)));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertex_positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vertex_normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vertex_uvs);
        mesh.insert_attribute(Mesh::ATTRIBUTE_TANGENT, vertex_tangents);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vertex_colors);
        mesh
    }
}

#[derive(TypeUuid, AsBindGroup, Clone, Default, Debug)]
#[uuid = "dfaf9f57-5ecc-4071-8d64-bb0edd4cc238"]
#[bind_group_data(EffectMaterialKey)]
pub struct LightningEffectMaterial {
    #[uniform(0)]
    pub uv_transform: Vec4,
    #[uniform(0)]
    pub color: Color,
}
impl Material for LightningEffectMaterial {
    fn vertex_shader() -> ShaderRef { "shaders/default_effect.wgsl".into() }
    fn fragment_shader() -> ShaderRef { "shaders/effect_lightning.wgsl".into() }
    fn alpha_mode(&self) -> AlphaMode { AlphaMode::Premultiplied }
    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline<Self>,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        _layout: &bevy::render::mesh::MeshVertexBufferLayout,
        key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        key.bind_group_data.apply(descriptor);
        Ok(())
    }
}
impl From<&LightningEffectMaterial> for EffectMaterialKey {
    fn from(_value: &LightningEffectMaterial) -> Self { Self {
        billboard: Some(Billboard::Axial),
        color_mask: true,
        ..Default::default()
    } }
}
