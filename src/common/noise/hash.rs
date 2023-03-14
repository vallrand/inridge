const PRIME_X: i32 = 0x1dde90c9;
const PRIME_Y: i32 = 0x43c42e4d;
const PRIME_Z: i32 = 0x668b6e2f;
const PRIME_W: i32 = 0x208b7487;

#[inline]
pub fn hash11(x: i32) -> i32 {
    let mut hash: u32 = (x ^ x >> 16) as u32;
    hash = hash.wrapping_mul(0x7feb352d);
    hash = hash ^ hash >> 15;
    hash = hash.wrapping_mul(0x846ca68b);
    hash = hash ^ hash >> 16;
    hash as i32
}
#[inline]
pub fn hash11_reverse(x: i32) -> i32 {
    let mut hash: u32 = (x ^ x >> 16) as u32;
    hash = hash.wrapping_mul(0x43021123);
    hash = hash ^ hash >> 15 ^ hash >> 30;
    hash = hash.wrapping_mul(0x1d69e2a5);
    hash = hash ^ hash >> 16;
    hash as i32
}
#[inline]
pub fn hash21(x: i32, y: i32, seed: u32) -> i32 {
    hash11(seed as i32 ^ x.wrapping_mul(PRIME_X) ^ y.wrapping_mul(PRIME_Y))
}
#[inline]
pub fn hash31(x: i32, y: i32, z: i32, seed: u32) -> i32 {
    hash11(seed as i32 ^ x.wrapping_mul(PRIME_X) ^ y.wrapping_mul(PRIME_Y) ^ z.wrapping_mul(PRIME_Z))
}
#[inline]
pub fn hash41(x: i32, y: i32, z: i32, w: i32, seed: u32) -> i32 {
    hash11(seed as i32 ^ x.wrapping_mul(PRIME_X) ^ y.wrapping_mul(PRIME_Y) ^ z.wrapping_mul(PRIME_Z) ^ w.wrapping_mul(PRIME_W))
}
pub trait UniformRange { fn normalize(&self) -> f32; }
impl UniformRange for i32 {
    #[inline] fn normalize(&self) -> f32 { *self as f32 / (std::i32::MAX as f32 + 1.0) }
}

pub fn gaussian(mean: f32, std: f32, random: impl Fn() -> f32) -> f32 {
    let u = 1.0 - random();
    let v = random();
    let z = (-2.0 * u.ln()).sqrt() * (2.0 * std::f32::consts::PI * v).cos();
    z * std + mean
}

pub fn random_vec3(random: impl Fn() -> f32) -> bevy::math::Vec3A {
    let theta = (2.0 * random() - 1.0).acos();
    let phi = 2.0 * random() * std::f32::consts::PI;
    bevy::math::Vec3A::new(phi.cos() * theta.sin(), phi.sin() * theta.sin(), theta.cos())
}

#[derive(Clone, Default)]
pub struct MurMurHash {
    state: u64
}
impl MurMurHash {
    pub fn from_seed(seed: u64) -> Self { Self { state: seed } }
    pub fn next(&mut self, seed: u64) -> u64 {
        let mut hash: u64 = self.state;
        hash ^= seed;
        hash ^= hash.wrapping_shr(16);
        hash = hash.wrapping_mul(0x85ebca6b);
        hash ^= hash.wrapping_shr(13);
        hash = hash.wrapping_mul(0xc2b2ae35);
        hash ^= hash.wrapping_shr(16);
        self.state = hash;
        hash
    }
    pub fn next_f32(&mut self) -> f32 {
        self.next(u64::MAX) as f32 / (u64::MAX - 1) as f32
    }
    pub fn next_u32(&mut self, max: u32) -> u32 {
        (self.next_f32() * max as f32) as u32
    }
}