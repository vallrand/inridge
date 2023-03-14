use bevy::prelude::*;
use bevy::math::{Vec3A, Quat, Mat4, Vec4Swizzles};
use super::{Overlap,aabb::AABB};

#[derive(Copy, Clone, Default, PartialEq, Debug)]
pub struct Ray {
    pub origin: Vec3A,
    pub direction: Vec3A,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {Self {
        origin: origin.into(),
        direction: direction.normalize().into(),
    }}
    pub fn from_transform(transform: Mat4) -> Self {
        let target = transform.project_point3(Vec3::NEG_Z);
        let origin = transform.w_axis.xyz();
        Self::new(origin, target - origin)
    }
    pub fn from_screenspace(view_transform: Mat4, camera: &Camera, cursor: Option<Vec2>) -> Option<Self> { 
        let cursor_ndc = if let Some(cursor) = cursor {
            let (viewport_min, viewport_max) = camera.logical_viewport_rect()?;
            let screen_size = camera.logical_target_size()?;
            let normalized = (cursor - Vec2::new(viewport_min.x, screen_size.y - viewport_max.y)) / (viewport_max - viewport_min);
            normalized * 2.0 - 1.0
        } else {
            Vec2::ZERO
        };

        let projection = camera.projection_matrix();
        let far_ndc = projection.project_point3(Vec3::NEG_Z).z;
        let near_ndc = projection.project_point3(Vec3::Z).z;
        let ndc_to_world: Mat4 = view_transform * projection.inverse();
        let near = ndc_to_world.project_point3(cursor_ndc.extend(near_ndc));
        let far = ndc_to_world.project_point3(cursor_ndc.extend(far_ndc));
        Some(Self::new(near, far - near))
    }
    #[inline] pub fn as_mat4(&self, up: Vec3A) -> Mat4 {
        Mat4::from_rotation_translation(
            Quat::from_axis_angle(up.cross(self.direction).into(), up.dot(self.direction).acos()),
            self.origin.into()
        )
    }
    ///https://www.scratchapixel.com/lessons/3d-basic-rendering/ray-tracing-rendering-a-triangle/moller-trumbore-ray-triangle-intersection.html
    pub fn raycast_moller_trumbore(&self, triangle: [Vec3A; 3], cull_backfaces: bool) -> Option<(f32,f32,f32)> {
        let edge_01 = triangle[1] - triangle[0];
        let edge_02 = triangle[2] - triangle[0];
        let normal = self.direction.cross(edge_02);
        let determinant = edge_01.dot(normal);
        if determinant.abs() < f32::EPSILON && determinant < 0.0 && cull_backfaces {
            return None;
        }
        let determinant_inverse = 1.0 / determinant;

        let tvec = self.origin - triangle[0];
        let u = tvec.dot(normal) * determinant_inverse;
        if u < 0.0 || u > 1.0 { return None; }

        let qvec = tvec.cross(edge_01);
        let v = self.direction.dot(qvec) * determinant_inverse;
        if v < 0.0 || u + v > 1.0 { return None; }

        let distance = edge_02.dot(qvec) * determinant_inverse;
        if distance > 0.0 {
            Some((distance, u, v))
        } else {
            None
        }
    }
}

impl std::ops::Mul<Ray> for Mat4 {
    type Output = Ray;
    #[inline] fn mul(self, rhs: Ray) -> Self::Output { Self::Output {
        origin: self.transform_point3a(rhs.origin),
        direction: self.transform_vector3a(rhs.direction).normalize(),
    } }
}

impl Overlap<&AABB> for Ray {
    #[inline] fn overlap(&self, aabb: &AABB, epsilon: f32) -> bool {
        let hit_min = (aabb.min - self.origin) / self.direction;
        let hit_max = (aabb.max - self.origin) / self.direction;
        let min = hit_min.min(hit_max);
        let max = hit_min.max(hit_max);

        let near = min.x.max(min.y).max(min.z);
        let far = max.x.min(max.y).min(max.z);

        near < far && far > 0.0
    }
}