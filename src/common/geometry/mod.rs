use bevy::math::{Vec3A,Mat3A};

pub trait GeometryTopology {
    fn vertices(&self) -> &[Vec3A];
    fn indices(&self) -> &[usize];
}

pub struct MeshGeometry {
    pub vertices: Vec<Vec3A>,
    pub indices: Vec<usize>
}
impl MeshGeometry {
    pub fn invert(&mut self){
        for i in 0..self.indices.len() / 3 {
            self.indices.swap(i * 3 + 0, i * 3 + 2);
        }
    }
}
impl GeometryTopology for MeshGeometry {
    fn vertices(&self) -> &[Vec3A] { &self.vertices }
    fn indices(&self) -> &[usize] { &self.indices }
}
impl std::ops::MulAssign<Mat3A> for MeshGeometry {
    fn mul_assign(&mut self, rhs: Mat3A) {
        self.vertices.iter_mut()
        .for_each(|vertex|*vertex = rhs * vertex.clone());
    }
}

mod helper;
pub use helper::*;

pub mod sphere;
pub mod quad_sphere;
pub mod plane;
pub mod subdivide;
pub mod cylinder;
pub mod cube;

mod unwrap;
pub use unwrap::*;

mod icosahedron;
pub use icosahedron::Icosahedron;
