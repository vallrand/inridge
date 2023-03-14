use bevy::math::Vec3A;
use super::GeometryTopology;

///12 vertices, 20 faces, 30 edges
#[derive(Default, Copy, Clone, Debug)]
pub struct Icosahedron;
impl GeometryTopology for Icosahedron {
    #[inline] fn vertices(&self) -> &[Vec3A] { &VERTICES }
    #[inline] fn indices(&self) -> &[usize] { &INDICES }
}
impl Icosahedron {
    pub fn circumscribed_tile_radius(subdivisions: usize) -> f32 {
        let edge_angle: f32 = 2.0*f32::asin(0.5/f32::sin(2.0*std::f32::consts::PI/5.0));
        2.0 * f32::sin(0.5 * edge_angle / (subdivisions as f32 + 1.0))
    }
}

const GOLDEN_RATIO: f32 = 1.618033988749894;
///sqrt(1 / (1 + GOLDEN_RATIO * GOLDEN_RATIO))
const SIDE: f32 = 0.5257311121191338;
const VERTICES: [Vec3A; 12] = [
    Vec3A::new(SIDE, GOLDEN_RATIO * SIDE, 0.0),
    Vec3A::new(-SIDE, GOLDEN_RATIO * SIDE, 0.0),
    Vec3A::new(SIDE,-GOLDEN_RATIO * SIDE,0.0),
    Vec3A::new(-SIDE,-GOLDEN_RATIO * SIDE,0.0),
    Vec3A::new(0.0,SIDE,GOLDEN_RATIO * SIDE),
    Vec3A::new(0.0,-SIDE,GOLDEN_RATIO * SIDE),
    Vec3A::new(0.0,SIDE,-GOLDEN_RATIO * SIDE),
    Vec3A::new(0.0,-SIDE,-GOLDEN_RATIO * SIDE),
    Vec3A::new(GOLDEN_RATIO * SIDE,0.0,SIDE),
    Vec3A::new(-GOLDEN_RATIO * SIDE,0.0,SIDE),
    Vec3A::new(GOLDEN_RATIO * SIDE,0.0,-SIDE),
    Vec3A::new(-GOLDEN_RATIO * SIDE,0.0,-SIDE),
];
const INDICES: [usize; 20 * 3] = [
    0,1,4,
    1,9,4,
    4,9,5,
    5,9,3,
    2,3,7,
    3,2,5,
    7,10,2,
    0,8,10,
    0,4,8,
    8,2,10,
    8,4,5,
    8,5,2,
    1,0,6,
    11,1,6,
    3,9,11,
    6,10,7,
    3,11,7,
    11,6,7,
    6,0,10,
    9,1,11,
];

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::super::adjacency::Graph;
    use super::super::subdivide::{slerp, subdivide};
    #[test] fn subdivide_hexsphere() {
        let vertex_count = vec![12 + 30, 12 + 80, 12 + 150, 12 + 240, 12 + 350];
        for subdivisions in 1..=5 {
            let hexsphere = subdivide(&Icosahedron{}, subdivisions, slerp);
            let graph = Graph::from(&hexsphere);

            assert_eq!(hexsphere.vertices.len(), vertex_count[subdivisions - 1]);

            let mut min: f32 = f32::MAX;
            assert_eq!((0..hexsphere.vertices.len()).filter(|&i| {
                let neighbors = graph.neighbors(i).unwrap_or_default().clone();

                for k in 0..neighbors.len() {
                    let d = (hexsphere.vertices[neighbors[k]] - hexsphere.vertices[i]).length();
                    min = min.min(d);
                }

                assert!(neighbors.len() >= 5 && neighbors.len() <= 6, "5 <= {} <= 6", neighbors.len());
                neighbors.len() == 5
            }).count(), 12);
            assert!((min - Icosahedron::circumscribed_tile_radius(subdivisions)).abs() < 1e-6, "{} != {}", min, Icosahedron::circumscribed_tile_radius(subdivisions))
        }
    }
}