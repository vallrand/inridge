use bevy::math::{Vec2,Vec3A};
use super::sampler::HeightSampler;
use super::helpers::*;
use super::hash::*;

pub struct ValueNoise {
    interpolation: Interpolation
}
impl Default for ValueNoise {
    fn default() -> Self { Self { interpolation: Interpolation::Hermite } }
}
impl HeightSampler<Vec2> for ValueNoise {
    fn sample(&self, coordinate: Vec2, seed: u32) -> f32 {
        let i0 = coordinate.floor().as_ivec2();
        let i1 = i0 + 1;

        let xs = self.interpolation.curve(coordinate.x - i0.x as f32);
        let ys = self.interpolation.curve(coordinate.y - i0.y as f32);

        lerp(
            lerp(hash21(i0.x, i0.y, seed).normalize(),hash21(i1.x, i0.y, seed).normalize(),xs),
            lerp(hash21(i0.x, i1.y, seed).normalize(),hash21(i1.x, i1.y, seed).normalize(),xs),
        ys)
    }
}
impl HeightSampler<Vec3A> for ValueNoise {
    fn sample(&self, coordinate: Vec3A, seed: u32) -> f32 {
        let i0 = coordinate.floor().as_ivec3();
        let i1 = i0 + 1;

        let xs = self.interpolation.curve(coordinate.x - i0.x as f32);
        let ys = self.interpolation.curve(coordinate.y - i0.y as f32);
        let zs = self.interpolation.curve(coordinate.z - i0.z as f32);

        lerp(
            lerp(
                lerp(hash31(i0.x, i0.y, i0.z, seed).normalize(), hash31(i1.x, i0.y, i0.z, seed).normalize(), xs),
                lerp(hash31(i0.x, i1.y, i0.z, seed).normalize(), hash31(i1.x, i1.y, i0.z, seed).normalize(), xs),
            ys),
            lerp(
                lerp(hash31(i0.x, i0.y, i1.z, seed).normalize(), hash31(i1.x, i0.y, i1.z, seed).normalize(), xs),
                lerp(hash31(i0.x, i1.y, i1.z, seed).normalize(), hash31(i1.x, i1.y, i1.z, seed).normalize(), xs),
            ys),
        zs)
    }
}


pub struct ValueCubicNoise {
    interpolation: Interpolation
}
impl Default for ValueCubicNoise {
    fn default() -> Self { Self { interpolation: Interpolation::Hermite } }
}
impl HeightSampler<Vec2> for ValueCubicNoise {
    fn sample(&self, coordinate: Vec2, seed: u32) -> f32 {
        let i1 = coordinate.floor().as_ivec2();
        let i0 = i1 - 1;
        let i2 = i1 + 1;
        let i3 = i1 + 2;

        let xs = self.interpolation.curve(coordinate.x - i1.x as f32);
        let ys = self.interpolation.curve(coordinate.y - i1.y as f32);

        const IW: f32 = 1.0 / (1.5 * 1.5);
        IW * cubic_lerp(
            cubic_lerp(hash21(i0.x, i0.y, seed).normalize(), hash21(i1.x, i0.y, seed).normalize(), hash21(i2.x, i0.y, seed).normalize(), hash21(i3.x, i0.y, seed).normalize(),
            xs),
            cubic_lerp(hash21(i0.x, i1.y, seed).normalize(), hash21(i1.x, i1.y, seed).normalize(), hash21(i2.x, i1.y, seed).normalize(), hash21(i3.x, i1.y, seed).normalize(),
            xs),
            cubic_lerp(hash21(i0.x, i2.y, seed).normalize(), hash21(i1.x, i2.y, seed).normalize(), hash21(i2.x, i2.y, seed).normalize(), hash21(i3.x, i2.y, seed).normalize(),
            xs),
            cubic_lerp(hash21(i0.x, i3.y, seed).normalize(), hash21(i1.x, i3.y, seed).normalize(), hash21(i2.x, i3.y, seed).normalize(), hash21(i3.x, i3.y, seed).normalize(),
            xs),
        ys)
    }
}

impl HeightSampler<Vec3A> for ValueCubicNoise {
    fn sample(&self, coordinate: Vec3A, seed: u32) -> f32 {
        let i1 = coordinate.floor().as_ivec3();
        let i0 = i1 - 1;
        let i2 = i1 + 1;
        let i3 = i1 + 2;

        let xs = self.interpolation.curve(coordinate.x - i1.x as f32);
        let ys = self.interpolation.curve(coordinate.y - i1.y as f32);
        let zs = self.interpolation.curve(coordinate.z - i1.z as f32);

        const IW: f32 = 1.0 / (1.5 * 1.5 * 1.5);
        IW * cubic_lerp(
            cubic_lerp(
            cubic_lerp(hash31(i0.x, i0.y, i0.z, seed).normalize(), hash31(i1.x, i0.y, i0.z, seed).normalize(), hash31(i2.x, i0.y, i0.z, seed).normalize(), hash31(i3.x, i0.y, i0.z, seed).normalize(), xs),
            cubic_lerp(hash31(i0.x, i1.y, i0.z, seed).normalize(), hash31(i1.x, i1.y, i0.z, seed).normalize(), hash31(i2.x, i1.y, i0.z, seed).normalize(), hash31(i3.x, i1.y, i0.z, seed).normalize(), xs),
            cubic_lerp(hash31(i0.x, i2.y, i0.z, seed).normalize(), hash31(i1.x, i2.y, i0.z, seed).normalize(), hash31(i2.x, i2.y, i0.z, seed).normalize(), hash31(i3.x, i2.y, i0.z, seed).normalize(), xs),
            cubic_lerp(hash31(i0.x, i3.y, i0.z, seed).normalize(), hash31(i1.x, i3.y, i0.z, seed).normalize(), hash31(i2.x, i3.y, i0.z, seed).normalize(), hash31(i3.x, i3.y, i0.z, seed).normalize(), xs),
            ys),
            cubic_lerp(
            cubic_lerp(hash31(i0.x, i0.y, i1.z, seed).normalize(), hash31(i1.x, i0.y, i1.z, seed).normalize(), hash31(i2.x, i0.y, i1.z, seed).normalize(), hash31(i3.x, i0.y, i1.z, seed).normalize(), xs),
            cubic_lerp(hash31(i0.x, i1.y, i1.z, seed).normalize(), hash31(i1.x, i1.y, i1.z, seed).normalize(), hash31(i2.x, i1.y, i1.z, seed).normalize(), hash31(i3.x, i1.y, i1.z, seed).normalize(), xs),
            cubic_lerp(hash31(i0.x, i2.y, i1.z, seed).normalize(), hash31(i1.x, i2.y, i1.z, seed).normalize(), hash31(i2.x, i2.y, i1.z, seed).normalize(), hash31(i3.x, i2.y, i1.z, seed).normalize(), xs),
            cubic_lerp(hash31(i0.x, i3.y, i1.z, seed).normalize(), hash31(i1.x, i3.y, i1.z, seed).normalize(), hash31(i2.x, i3.y, i1.z, seed).normalize(), hash31(i3.x, i3.y, i1.z, seed).normalize(), xs),
            ys),
            cubic_lerp(
            cubic_lerp(hash31(i0.x, i0.y, i2.z, seed).normalize(), hash31(i1.x, i0.y, i2.z, seed).normalize(), hash31(i2.x, i0.y, i2.z, seed).normalize(), hash31(i3.x, i0.y, i2.z, seed).normalize(), xs),
            cubic_lerp(hash31(i0.x, i1.y, i2.z, seed).normalize(), hash31(i1.x, i1.y, i2.z, seed).normalize(), hash31(i2.x, i1.y, i2.z, seed).normalize(), hash31(i3.x, i1.y, i2.z, seed).normalize(), xs),
            cubic_lerp(hash31(i0.x, i2.y, i2.z, seed).normalize(), hash31(i1.x, i2.y, i2.z, seed).normalize(), hash31(i2.x, i2.y, i2.z, seed).normalize(), hash31(i3.x, i2.y, i2.z, seed).normalize(), xs),
            cubic_lerp(hash31(i0.x, i3.y, i2.z, seed).normalize(), hash31(i1.x, i3.y, i2.z, seed).normalize(), hash31(i2.x, i3.y, i2.z, seed).normalize(), hash31(i3.x, i3.y, i2.z, seed).normalize(), xs),
            ys),
            cubic_lerp(
            cubic_lerp(hash31(i0.x, i0.y, i3.z, seed).normalize(), hash31(i1.x, i0.y, i3.z, seed).normalize(), hash31(i2.x, i0.y, i3.z, seed).normalize(), hash31(i3.x, i0.y, i3.z, seed).normalize(), xs),
            cubic_lerp(hash31(i0.x, i1.y, i3.z, seed).normalize(), hash31(i1.x, i1.y, i3.z, seed).normalize(), hash31(i2.x, i1.y, i3.z, seed).normalize(), hash31(i3.x, i1.y, i3.z, seed).normalize(), xs),
            cubic_lerp(hash31(i0.x, i2.y, i3.z, seed).normalize(), hash31(i1.x, i2.y, i3.z, seed).normalize(), hash31(i2.x, i2.y, i3.z, seed).normalize(), hash31(i3.x, i2.y, i3.z, seed).normalize(), xs),
            cubic_lerp(hash31(i0.x, i3.y, i3.z, seed).normalize(), hash31(i1.x, i3.y, i3.z, seed).normalize(), hash31(i2.x, i3.y, i3.z, seed).normalize(), hash31(i3.x, i3.y, i3.z, seed).normalize(), xs),
            ys),
        zs)
    }
}