use bevy::math::{Vec2,Vec3A,Vec4};
use super::sampler::HeightSampler;
use super::helpers::*;
use super::hash::{hash21,hash31,hash41};

pub fn gradient_dot_2d(x: i32, y: i32, xd: f32, yd: f32, seed: u32) -> f32 {
    let vector = GRADIENTS_2D[(hash21(x, y, seed) & 0b1111) as usize];
    xd * vector.x + yd * vector.y
}
pub fn gradient_dot_3d(x: i32, y: i32, z: i32, xd: f32, yd: f32, zd: f32, seed: u32) -> f32 {
    let vector = GRADIENTS_3D[(hash31(x, y, z, seed) & 0b1111) as usize];
    xd * vector.x + yd * vector.y + zd * vector.z
}
pub fn gradient_dot_4d(x: i32, y: i32, z: i32, w: i32, xd: f32, yd: f32, zd: f32, wd: f32, seed: u32) -> f32 {
    let vector = GRADIENTS_4D[(hash41(x, y, z, w, seed) & 0b11111) as usize];
    xd * vector.x + yd * vector.y + zd * vector.z + wd * vector.w
}

pub struct PerlinNoise {
    interpolation: Interpolation
}
impl Default for PerlinNoise {
    fn default() -> Self {Self { interpolation: Interpolation::Quintic }}
}

impl HeightSampler<Vec2> for PerlinNoise {
    fn sample(&self, coordinate: Vec2, seed: u32) -> f32 {
        let i0 = coordinate.floor().as_ivec2();
        let i1 = i0 + 1;

        let d0 = coordinate - i0.as_vec2();
        let d1 = d0 - 1.0;

        let xs = self.interpolation.curve(d0.x);
        let ys = self.interpolation.curve(d0.y);

        ///= 1 / (0.5 * dimension * sqrt(unit_length/dimension))
        const INV_RANGE: f32 = 1.414213562373095;
        INV_RANGE * lerp(lerp(
            gradient_dot_2d(i0.x, i0.y, d0.x, d0.y, seed),
            gradient_dot_2d(i1.x, i0.y, d1.x, d0.y, seed),
            xs,
        ), lerp(
            gradient_dot_2d(i0.x, i1.y, d0.x, d1.y, seed),
            gradient_dot_2d(i1.x, i1.y, d1.x, d1.y, seed),
            xs,
        ), ys)
    }
}

impl HeightSampler<Vec3A> for PerlinNoise {
    fn sample(&self, coordinate: Vec3A, seed: u32) -> f32 {
        let i0 = coordinate.floor().as_ivec3();
        let i1 = i0 + 1;

        let d0 = coordinate - i0.as_vec3a();
        let d1 = d0 - 1.0;

        let xs = self.interpolation.curve(d0.x);
        let ys = self.interpolation.curve(d0.y);
        let zs = self.interpolation.curve(d0.z);

        /// 1 / (0.5 * dimension * sqrt(unit_length/dimension))
        const INV_RANGE: f32 = 1.1547005383792517;
        INV_RANGE * lerp(
            lerp(
                lerp(
                    gradient_dot_3d(i0.x, i0.y, i0.z, d0.x, d0.y, d0.z, seed),
                    gradient_dot_3d(i1.x, i0.y, i0.z, d1.x, d0.y, d0.z, seed),
                 xs),
                lerp(
                    gradient_dot_3d(i0.x, i1.y, i0.z, d0.x, d1.y, d0.z, seed),
                    gradient_dot_3d(i1.x, i1.y, i0.z, d1.x, d1.y, d0.z, seed),
                 xs),
            ys),
            lerp(
                lerp(
                    gradient_dot_3d(i0.x, i0.y, i1.z, d0.x, d0.y, d1.z, seed),
                    gradient_dot_3d(i1.x, i0.y, i1.z, d1.x, d0.y, d1.z, seed),
                 xs),
                lerp(
                    gradient_dot_3d(i0.x, i1.y, i1.z, d0.x, d1.y, d1.z, seed),
                    gradient_dot_3d(i1.x, i1.y, i1.z, d1.x, d1.y, d1.z, seed),
                 xs),
             ys),
        zs)
    }
}

const GRADIENTS_2D: [Vec2; 16] = [
    Vec2::new(0.98078525, 0.19509032),
    Vec2::new(0.8314696, 0.55557024),
    Vec2::new(0.5555702, 0.83146966),
    Vec2::new(0.19509023, 0.9807853),
    Vec2::new(-0.19509032, 0.98078525),
    Vec2::new(-0.55557036, 0.83146954),
    Vec2::new(-0.83146966, 0.5555702),
    Vec2::new(-0.9807853, 0.19509031),
    Vec2::new(-0.98078525, -0.19509049),
    Vec2::new(-0.83146954, -0.5555703),
    Vec2::new(-0.55557, -0.8314698),
    Vec2::new(-0.19509038, -0.98078525),
    Vec2::new(0.19509041, -0.98078525),
    Vec2::new(0.5555704, -0.8314695),
    Vec2::new(0.8314696, -0.5555703),
    Vec2::new(0.9807853, -0.19509023),
];

const GRADIENTS_3D: [Vec3A; 16] = [
    Vec3A::new(0.9238795, -1.6727626e-8, 0.38268346),
    Vec3A::new(0.9238795, 4.5634545e-9, -0.38268346),
    Vec3A::new(0.38268343, 0.8001031, 0.46193975),
    Vec3A::new(0.38268343, -4.0384055e-8, 0.9238795),
    Vec3A::new(0.38268343, -0.8001032, 0.46193957),
    Vec3A::new(0.38268343, -0.800103, -0.46193993),
    Vec3A::new(0.38268343, 1.10171525e-8, -0.9238795),
    Vec3A::new(0.38268343, 0.80010325, -0.46193954),
    Vec3A::new(-0.38268352, 0.8001031, 0.46193975),
    Vec3A::new(-0.38268352, -4.0384055e-8, 0.9238795),
    Vec3A::new(-0.38268352, -0.8001032, 0.46193957),
    Vec3A::new(-0.38268352, -0.800103, -0.46193993),
    Vec3A::new(-0.38268352, 1.10171525e-8, -0.9238795),
    Vec3A::new(-0.38268352, 0.80010325, -0.46193954),
    Vec3A::new(-0.9238796, -1.6727617e-8, 0.38268328),
    Vec3A::new(-0.9238796, 4.5634523e-9, -0.38268328),
];

pub const SQRT_3: f32 = 1.7320508075688772;
const GRADIENTS_4D: [Vec4; 32] = [
    Vec4::new(SQRT_3,SQRT_3,SQRT_3,0.),
    Vec4::new(SQRT_3,SQRT_3,-SQRT_3,0.),
    Vec4::new(SQRT_3,-SQRT_3,SQRT_3,0.),
    Vec4::new(SQRT_3,-SQRT_3,-SQRT_3,0.),
    Vec4::new(-SQRT_3,SQRT_3,SQRT_3,0.),
    Vec4::new(-SQRT_3,SQRT_3,-SQRT_3,0.),
    Vec4::new(-SQRT_3,-SQRT_3,SQRT_3,0.),
    Vec4::new(-SQRT_3,-SQRT_3,-SQRT_3,0.),
    Vec4::new(SQRT_3,SQRT_3,0.,SQRT_3),
    Vec4::new(SQRT_3,SQRT_3,0.,-SQRT_3),
    Vec4::new(SQRT_3,-SQRT_3,0.,SQRT_3),
    Vec4::new(SQRT_3,-SQRT_3,0.,-SQRT_3),
    Vec4::new(-SQRT_3,SQRT_3,0.,SQRT_3),
    Vec4::new(-SQRT_3,SQRT_3,0.,-SQRT_3),
    Vec4::new(-SQRT_3,-SQRT_3,0.,SQRT_3),
    Vec4::new(-SQRT_3,-SQRT_3,0.,-SQRT_3),
    Vec4::new(SQRT_3,0.,SQRT_3,SQRT_3),
    Vec4::new(SQRT_3,0.,SQRT_3,-SQRT_3),
    Vec4::new(SQRT_3,0.,-SQRT_3,SQRT_3),
    Vec4::new(SQRT_3,0.,-SQRT_3,-SQRT_3),
    Vec4::new(-SQRT_3,0.,SQRT_3,SQRT_3),
    Vec4::new(-SQRT_3,0.,SQRT_3,-SQRT_3),
    Vec4::new(-SQRT_3,0.,-SQRT_3,SQRT_3),
    Vec4::new(-SQRT_3,0.,-SQRT_3,-SQRT_3),
    Vec4::new(0.,SQRT_3,SQRT_3,SQRT_3),
    Vec4::new(0.,SQRT_3,SQRT_3,-SQRT_3),
    Vec4::new(0.,SQRT_3,-SQRT_3,SQRT_3),
    Vec4::new(0.,SQRT_3,-SQRT_3,-SQRT_3),
    Vec4::new(0.,-SQRT_3,SQRT_3,SQRT_3),
    Vec4::new(0.,-SQRT_3,SQRT_3,-SQRT_3),
    Vec4::new(0.,-SQRT_3,-SQRT_3,SQRT_3),
    Vec4::new(0.,-SQRT_3,-SQRT_3,-SQRT_3),
];