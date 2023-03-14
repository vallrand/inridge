use bevy::{
    prelude::*,
    math::{Vec3A},
    render::mesh::Indices,
    render::render_resource::PrimitiveTopology
};
use super::ray::Ray;
use super::Intersect;
use super::super::geometry::{plane::Plane,sphere::Sphere};

#[derive(Clone, Copy, Default)]
pub struct RaycastHit {
    pub distance: f32,
    pub index: usize,
    pub position: Vec3A,
    pub normal: Vec3A,
}

impl std::ops::MulAssign<Mat4> for RaycastHit {
    fn mul_assign(&mut self, rhs: Mat4) {
        self.position = rhs.transform_point3a(self.position);
        self.normal = rhs.transform_vector3a(self.normal).normalize();
    }
}

impl Intersect<&Plane> for Ray {
    type Output = RaycastHit;
    ///https://www.scratchapixel.com/lessons/3d-basic-rendering/minimal-ray-tracer-rendering-simple-shapes/ray-plane-and-ray-disk-intersection.html
    fn intersect(&self, &Plane { origin, normal }: &Plane) -> Option<Self::Output> {
        let dot = self.direction.dot(normal);
        if dot.abs() > f32::EPSILON {
            let distance = normal.dot(origin - self.origin) / dot;
            if distance > 0.0 {
                Some(RaycastHit { distance, index: 0, position: self.origin + self.direction * distance, normal })
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl Intersect<&Sphere> for Ray {
    type Output = RaycastHit;
    ///https://www.scratchapixel.com/lessons/3d-basic-rendering/minimal-ray-tracer-rendering-simple-shapes/ray-sphere-intersection.html
    fn intersect(&self, &Sphere { origin, radius }: &Sphere) -> Option<Self::Output> {
        let delta = origin - self.origin;
        let t = delta.dot(self.direction);
        let d2 = radius * radius - (delta.dot(delta) - t * t);
        if d2 < 0.0 { return None; }
        let half = d2.sqrt() * t.signum() * radius.signum();
        let near = t - half;
        let far = t + half;
        let distance = if near < 0.0 { far }else{ near };
        if distance >= 0.0 {
            let position = self.origin + self.direction * distance;
            Some(RaycastHit { distance, index: 0, position, normal: (position - origin).normalize() })
        } else {
            None
        }
    }
}

enum IndexArrayView<'a> {
    U16(&'a [u16]),
    U32(&'a [u32]),
    None(usize)
}
impl IndexArrayView<'_> {
    #[inline] pub fn len(&self) -> usize { match self {
        Self::U16(vec) => vec.len(),
        Self::U32(vec) => vec.len(),
        &Self::None(len) => len            
    } }
    #[inline] fn index(&self, index: usize) -> usize { match self {
        Self::U16(vec) => vec[index] as usize,
        Self::U32(vec) => vec[index] as usize,
        Self::None(_) => index
    } }
}

impl Intersect<&Mesh> for Ray {
    type Output = RaycastHit;
    fn intersect(&self, mesh: &Mesh) -> Option<Self::Output> {
        if mesh.primitive_topology() != PrimitiveTopology::TriangleList {
            return None;
        }

        let positions = mesh.attribute(Mesh::ATTRIBUTE_POSITION)?.as_float3()?;
        let normals = mesh.attribute(Mesh::ATTRIBUTE_NORMAL).and_then(|list|list.as_float3());
        let indices = mesh.indices().map_or(IndexArrayView::None(positions.len()),
        |indices|match indices {
            Indices::U16(vec) => IndexArrayView::U16(&vec),
            Indices::U32(vec) => IndexArrayView::U32(&vec),
        });

        let mut min_distance_squared: f32 = f32::MAX;
        let mut closest_hit: Option<RaycastHit> = None;
        for i in (0..indices.len()).step_by(3) {
            let (i0, i1, i2) = (indices.index(i), indices.index(i+1), indices.index(i+2));
            let triangle = [Vec3A::from(positions[i0]), Vec3A::from(positions[i1]), Vec3A::from(positions[i2])];
            if closest_hit.is_some() && triangle.iter().all(|&vertex| (vertex - self.origin).length_squared() > min_distance_squared) {
                continue;
            }
            let Some((distance, u, v)) = self.raycast_moller_trumbore(triangle, true) else { continue };
            let distance_squared = distance * distance;
            if distance_squared > min_distance_squared { continue; }

            let position = self.origin + self.direction * distance;
            let normal = if let Some(normals) = normals {
                (1.0 - u - v) * Vec3A::from(normals[i0]) +
                u * Vec3A::from(normals[i1]) +
                v * Vec3A::from(normals[i2]) 
            }else{
                Vec3A::cross(
                    triangle[1] - triangle[0],
                    triangle[2] - triangle[0]
                ).normalize()
            };
            min_distance_squared = distance_squared;
            closest_hit = Some(RaycastHit { distance, index: i/3, position, normal })
        }
        closest_hit
    }
}