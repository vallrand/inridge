use bevy::prelude::*;
use bevy::math::cubic_splines::{Bezier,CubicCurve,CardinalSpline};
use bevy::render::mesh::PrimitiveTopology;
use bevy::reflect::TypeUuid;
use bevy::pbr::MaterialPipeline;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use super::shared::{EffectMaterialKey, Billboard};

pub struct StripeMesh {
    curve: CubicCurve<Vec3>,
    width: f32,
    resolution: usize,
}
impl StripeMesh {
    pub fn from_path(path: &[Vec3]) -> Self { Self {
        curve: CardinalSpline::new(0.5, path).to_curve(),
        resolution: path.len() * 4, width: 1.0,
    } }
    pub fn from_arc(origin: &GlobalTransform, target: &GlobalTransform) -> Self {
        let origin_position = origin.translation();
        let target_position = target.translation();
        let target_control_point = target.transform_point(Vec3::Y * 1.0);

        let forward = origin.affine().transform_vector3(Vec3::X).normalize();
        let origin_control_point = origin_position + forward * forward.dot(target_position - origin_position);

        Self{ curve: Bezier::new([[
            origin_position,
            origin_control_point,
            target_control_point,
            target_position
        ]]).to_curve(), resolution: 4, width: 1.0 }
    }
    pub fn with_stroke(mut self, width: f32) -> Self { self.width = width; self }
    pub fn with_quality(mut self, quality: usize) -> Self { self.resolution = quality; self }
}
impl From<StripeMesh> for Mesh {
    fn from(value: StripeMesh) -> Self {
        let positions: Vec<Vec3> = value.curve.iter_positions(value.resolution).collect();
        let velocities: Vec<Vec3> = value.curve.iter_velocities(value.resolution).collect();

        let mut vertex_positions: Vec<[f32; 3]> = Vec::with_capacity(positions.len() * 2);
        let mut vertex_normals: Vec<[f32; 3]> = Vec::with_capacity(positions.len() * 2);
        let mut vertex_tangents: Vec<[f32; 4]> = Vec::with_capacity(positions.len() * 2);
        let mut vertex_uvs: Vec<[f32; 2]> = Vec::with_capacity(positions.len() * 2);

        for i in 0..positions.len() {
            let binormal = velocities[i].normalize();

            let tangent = Vec3::cross(binormal, binormal.any_orthonormal_vector());
            let normal = Vec3::cross(binormal, tangent);

            vertex_positions.extend_from_slice(&[
                (positions[i] + tangent * value.width).to_array(),
                (positions[i] - tangent * value.width).to_array(),
            ]);
            vertex_normals.extend_from_slice(&[normal.to_array();2]);
            vertex_tangents.extend_from_slice(&[tangent.extend(value.width).to_array();2]);

            let v = i as f32 / (positions.len() - 1) as f32;
            vertex_uvs.extend_from_slice(&[[0.0, v],[1.0, v]]);
        }

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleStrip);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertex_positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vertex_normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vertex_uvs);
        mesh.insert_attribute(Mesh::ATTRIBUTE_TANGENT, vertex_tangents);
        mesh
    }
}

#[derive(TypeUuid, AsBindGroup, Clone, Default, Debug)]
#[uuid = "0ddace2d-35b7-43e8-92f0-33494287b3a0"]
#[bind_group_data(EffectMaterialKey)]
pub struct ProjectileTrailMaterial {
    #[uniform(0)]
    pub color: Color,
    #[uniform(0)]
    pub head_color: Color,
    #[uniform(0)]
    pub uv_transform: Vec4,
    #[uniform(0)]
    pub vertical_fade: Vec2,
    #[uniform(0)]
    pub time_scale: f32,
    #[uniform(0)]
    pub iterations: i32,

    pub billboard: bool,
    pub blend_mode: AlphaMode,
}
impl From<&ProjectileTrailMaterial> for EffectMaterialKey {
    fn from(value: &ProjectileTrailMaterial) -> Self { Self {
        billboard: if value.billboard { Some(Billboard::Axial) }else{ None },
        ..Default::default()
    } }
}
impl Material for ProjectileTrailMaterial {
    fn vertex_shader() -> ShaderRef { "shaders/default_effect.wgsl".into() }
    fn fragment_shader() -> ShaderRef { "shaders/effect_trail.wgsl".into() }
    fn alpha_mode(&self) -> AlphaMode { self.blend_mode }
    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        _layout: &bevy::render::mesh::MeshVertexBufferLayout,
        key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        key.bind_group_data.apply(descriptor);
        Ok(())
    }
}