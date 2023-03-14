use bevy::math::Vec3A;

#[derive(Clone, Copy)]
pub struct Plane {
    pub origin: Vec3A,
    pub normal: Vec3A
}