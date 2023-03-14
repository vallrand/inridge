pub mod sphere;
pub mod aabb;
pub mod ray;
pub mod intersect;
pub mod morton;

pub mod bvh;

pub trait Contains<Rhs = Self> {
    fn contains(&self, rhs: Rhs, epsilon: f32) -> bool;
}
pub trait Overlap<Rhs = Self> {
    fn overlap(&self, rhs: Rhs, epsilon: f32) -> bool;
}
pub trait Intersect<Rhs = Self> {
    type Output;
    fn intersect(&self, rhs: Rhs) -> Option<Self::Output>;
}