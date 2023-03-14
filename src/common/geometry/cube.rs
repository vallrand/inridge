use bevy::math::Vec3A;
use super::GeometryTopology;

#[derive(Default, Copy, Clone, Debug)]
pub struct Cube;
impl GeometryTopology for Cube {
    #[inline]
    fn vertices(&self) -> &[Vec3A] { &VERTICES }
    #[inline]
    fn indices(&self) -> &[usize] { &INDICES }
}
///1.0 / sqrt(3.0)
const SIDE: f32 = 0.5773502691896258;
const VERTICES: [Vec3A; 8] = [
    Vec3A::new(-SIDE, -SIDE, -SIDE),
    Vec3A::new( SIDE, -SIDE, -SIDE),
    Vec3A::new(-SIDE,  SIDE, -SIDE),
    Vec3A::new( SIDE,  SIDE, -SIDE),

    Vec3A::new(-SIDE, -SIDE,  SIDE),
    Vec3A::new( SIDE, -SIDE,  SIDE),
    Vec3A::new(-SIDE,  SIDE,  SIDE),
    Vec3A::new( SIDE,  SIDE,  SIDE),
];
const INDICES: [usize; 12 * 3] = [
    0,2,3,
    0,3,1,
    2,7,3,
    2,6,7,
    4,2,0,
    4,6,2,
    1,5,4,
    1,4,0,
    1,7,5,
    1,3,7,
    5,6,4,
    5,7,6,
];