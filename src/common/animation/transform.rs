use bevy::prelude::*;
use super::AnimatedProperty;

pub struct TransformRotation(pub Quat);
impl From<Quat> for TransformRotation { fn from(value: Quat) -> Self { Self(value) } }
impl AnimatedProperty for TransformRotation {
    type Component = Transform;
    const DEFAULT: Self = TransformRotation(Quat::IDENTITY);
    fn lerp(&self, rhs: &Self, fraction: f32) -> Self { Self(self.0.slerp(rhs.0, fraction)) }
    fn blend(&self, weight: f32, target: &mut Self::Component) {
        target.rotation = Self(target.rotation).lerp(self, weight).0;
    }
}

pub struct TransformTranslation(pub Vec3);
impl From<Vec3> for TransformTranslation { fn from(value: Vec3) -> Self { Self(value) } }
impl AnimatedProperty for TransformTranslation {
    type Component = Transform;
    const DEFAULT: Self = TransformTranslation(Vec3::ZERO);
    fn lerp(&self, rhs: &Self, fraction: f32) -> Self { Self(self.0.lerp(rhs.0, fraction)) }
    fn blend(&self, weight: f32, target: &mut Self::Component) {
        target.translation = Self(target.translation).lerp(self, weight).0;
    }
}

pub struct TransformScale(pub Vec3);
impl From<Vec3> for TransformScale { fn from(value: Vec3) -> Self { Self(value) } }
impl AnimatedProperty for TransformScale {
    type Component = Transform;
    const DEFAULT: Self = TransformScale(Vec3::ZERO);
    fn lerp(&self, rhs: &Self, fraction: f32) -> Self { Self(self.0.lerp(rhs.0, fraction)) }
    fn blend(&self, weight: f32, target: &mut Self::Component) {
        target.scale = Self(target.scale).lerp(self, weight).0;
    }
}