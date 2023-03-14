use bevy::math::UVec3;
use bevy::prelude::{Deref, DerefMut};
use std::hash::Hash;

fn spread_bits64_3(mut x: u64) -> u64 {
    x = x & 0x1fffff;
    x = (x | x << 32) & 0x1f00000000ffff;
    x = (x | x << 16) & 0x1f0000ff0000ff;
    x = (x | x << 8) & 0x100f00f00f00f00f;
    x = (x | x << 4) & 0x10c30c30c30c30c3;
    x = (x | x << 2) & 0x1249249249249249;
    x
}

fn squeeze_bits64_3(mut x: u64) -> u64 {
    x = x & 0x1249249249249249;
    x = (x ^ x >> 2) & 0x10c30c30c30c30c3;
    x = (x ^ x >> 4) & 0x100f00f00f00f00f;
    x = (x ^ x >> 8) & 0x1f0000ff0000ff;
    x = (x ^ x >> 16) & 0x1f00000000ffff;
    x = (x ^ x >> 32) & 0x1fffff;
    x
}

fn spread_bits32_3(mut x: u32) -> u32 {
    x = x & 0b00000000000000000000011111111111;
    x = (x | x << 16) & 0b11111111000000000000000011111111;
    x = (x | x << 8) & 0b00001111000000001111000000001111;
    x = (x | x << 4) & 0b01000011000011000011000011000011;
    x = (x | x << 2) & 0b01001001001001001001001001001001;
    x
}

fn squeeze_bits32_3(mut x: u32) -> u32 {
    x = x & 0b01001001001001001001001001001001;
    x = (x | x >> 2) & 0b01000011000011000011000011000011;
    x = (x | x >> 4) & 0b00001111000000001111000000001111;
    x = (x | x >> 8) & 0b11111111000000000000000011111111;
    x = (x | x >> 16) & 0b00000000000000000000011111111111;
    x
}

#[derive(Deref, DerefMut, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct MortonCode<T>(pub T);

impl From<UVec3> for MortonCode<u32> {
    fn from(value: UVec3) -> Self { MortonCode(
        spread_bits32_3(value.x) |
        spread_bits32_3(value.y) << 1 |
        spread_bits32_3(value.z) << 2
    ) }
}
impl From<UVec3> for MortonCode<u64> {
    fn from(value: UVec3) -> Self { MortonCode(
        spread_bits64_3(value.x as u64) |
        spread_bits64_3(value.y as u64) << 1 |
        spread_bits64_3(value.z as u64) << 2
    ) }
}

impl From<MortonCode<u32>> for UVec3 {
    fn from(value: MortonCode<u32>) -> Self { UVec3::new(
        squeeze_bits32_3(value.0), 
        squeeze_bits32_3(value.0 >> 1), 
        squeeze_bits32_3(value.0 >> 2)
    ) }
}

impl From<MortonCode<u64>> for UVec3 {
    fn from(value: MortonCode<u64>) -> Self { UVec3::new(
        squeeze_bits64_3(value.0) as u32, 
        squeeze_bits64_3(value.0 >> 1) as u32, 
        squeeze_bits64_3(value.0 >> 2) as u32
    ) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] pub fn morton_code_3d(){
        assert_eq!(UVec3::new(1, 4, 7), UVec3::from(MortonCode::<u32>::from(UVec3::new(1, 4, 7))));
    }
}