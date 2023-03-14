use bevy::math::{Vec3, Vec3A};
use super::{Overlap,Contains};

#[derive(Clone, Copy, Debug)]
pub struct AABB {
    pub min: Vec3A,
    pub max: Vec3A
}

impl Default for AABB {
    fn default() -> Self { Self {
        min: Vec3A::splat(f32::INFINITY),
        max: Vec3A::splat(f32::NEG_INFINITY)
    } }
}

impl AABB {
    ///https://tavianator.com/2014/ellipsoid_bounding_boxes.html
    #[inline] pub fn from_bounding_sphere(bounding_sphere: &crate::common::geometry::sphere::Sphere, world_transform: &bevy::math::Affine3A) -> Self {
        let origin = world_transform.transform_point3a(bounding_sphere.origin);
        let scale = Vec3A::new(
            world_transform.matrix3.x_axis.length(),
            world_transform.matrix3.y_axis.length(),
            world_transform.matrix3.z_axis.length(),
        );
        Self::from_min_max(origin - scale, origin + scale)
    }
    #[inline] pub const fn from_min_max(min: Vec3A, max: Vec3A) -> Self { Self { min, max } }
    #[inline] pub fn relative_eq(&self, other: &AABB, epsilon: f32) -> bool {
        f32::abs(self.min.x - other.min.x) < epsilon && f32::abs(self.min.y - other.min.y) < epsilon &&
        f32::abs(self.min.z - other.min.z) < epsilon && f32::abs(self.max.x - other.max.x) < epsilon &&
        f32::abs(self.max.y - other.max.y) < epsilon && f32::abs(self.max.z - other.max.z) < epsilon
    }
    #[inline] pub fn size(&self) -> Vec3A { self.max - self.min }
    #[inline] pub fn center(&self) -> Vec3A { (self.min + self.max) / 2.0 }
    #[inline] pub fn is_empty(&self) -> bool {
        self.min.x > self.max.x || self.min.y > self.max.y || self.min.z > self.max.z
    }
    #[inline] pub fn surface_area(&self) -> f32 {
        let size = self.size();
        2.0 * (size.x * size.y + size.x * size.z + size.y * size.z)
    }
    #[inline] pub fn volume(&self) -> f32 {
        let size = self.size();
        size.x * size.y * size.z
    }
    #[inline] pub fn max_axis(&self) -> usize {
        let size = self.size();
        if size.x > size.y && size.x > size.z { 0 } else if size.y > size.z { 1 } else { 2 }
    }
}

impl Contains<&AABB> for AABB {
    #[inline] fn contains(&self, rhs: &AABB, epsilon: f32) -> bool {
        self.min.x - rhs.min.x < epsilon && rhs.max.x - self.max.x < epsilon &&
        self.min.y - rhs.min.y < epsilon && rhs.max.y - self.max.y < epsilon &&
        self.min.z - rhs.min.z < epsilon && rhs.max.z - self.max.z < epsilon
    }
}
impl Contains<&Vec3A> for AABB {
    #[inline] fn contains(&self, rhs: &Vec3A, epsilon: f32) -> bool {
        (rhs.x - self.min.x) > -epsilon && (rhs.x - self.max.x) < epsilon &&
        (rhs.y - self.min.y) > -epsilon && (rhs.y - self.max.y) < epsilon &&
        (rhs.z - self.min.z) > -epsilon && (rhs.z - self.max.z) < epsilon
    }
}

impl Overlap<&AABB> for AABB {
    #[inline] fn overlap(&self, rhs: &AABB, epsilon: f32) -> bool {
        self.min.x - rhs.max.x < epsilon && rhs.min.x - self.max.x < epsilon &&
        self.min.y - rhs.max.y < epsilon && rhs.min.y - self.max.y < epsilon &&
        self.min.z - rhs.max.z < epsilon && rhs.min.z - self.max.z < epsilon
    }
}

impl std::ops::Add<AABB> for AABB {
    type Output = AABB;
    #[inline] fn add(self, rhs: AABB) -> Self::Output {
        AABB::from_min_max(self.min.min(rhs.min), self.max.max(rhs.max))
    }
}
impl std::ops::AddAssign<AABB> for AABB {
    #[inline] fn add_assign(&mut self, rhs: AABB) {
        self.min = self.min.min(rhs.min);
        self.max = self.max.max(rhs.max);
    }
}
impl std::ops::Add<f32> for AABB {
    type Output = AABB;
    #[inline] fn add(self, rhs: f32) -> Self::Output {
        AABB::from_min_max(self.min - rhs, self.max + rhs)
    }
}
impl std::ops::AddAssign<f32> for AABB {
    #[inline] fn add_assign(&mut self, rhs: f32) {
        self.min -= rhs;
        self.max += rhs;
    }
}
impl std::ops::Add<Vec3A> for AABB {
    type Output = AABB;
    #[inline] fn add(self, rhs: Vec3A) -> Self::Output {
        AABB::from_min_max(self.min.min(rhs), self.max.max(rhs))
    }
}
impl std::ops::AddAssign<Vec3A> for AABB {
    #[inline] fn add_assign(&mut self, rhs: Vec3A) {
        self.min = self.min.min(rhs);
        self.max = self.max.max(rhs)
    }
}
impl std::ops::AddAssign<Vec3> for AABB {
    #[inline] fn add_assign(&mut self, rhs: Vec3) {
        self.min = self.min.min(rhs.into());
        self.max = self.max.max(rhs.into())
    }
}

impl From<Vec3A> for AABB {
    #[inline] fn from(value: Vec3A) -> Self { Self::from_min_max(value, value) }
}
impl From<Vec3> for AABB {
    #[inline] fn from(value: Vec3) -> Self { Self::from_min_max(value.into(), value.into()) }
}

use bevy::render::primitives::Aabb;
impl From<&Aabb> for AABB {
    #[inline] fn from(aabb: &Aabb) -> Self { Self::from_min_max(aabb.min(), aabb.max()) }
}
