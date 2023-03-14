use bevy::math::Vec3A;
use std::hash::{Hasher,BuildHasherDefault};
use std::collections::{HashMap};
use super::{GeometryTopology,MeshGeometry};

#[derive(Default)]
pub struct PassThroughHasher(u64);
impl Hasher for PassThroughHasher {
    #[inline]
    fn finish(&self) -> u64 { self.0 }
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes.iter().rev() {
            self.0 = self.0.wrapping_shl(8) + (*byte as u64);
        }
    }
}

///spherical interpolation along the great arc
pub fn slerp(a: Vec3A, b: Vec3A) -> impl Fn(f32) -> Vec3A {
    let angle = a.dot(b).acos();
    let sin = angle.sin().recip();
    move |fraction: f32| -> Vec3A {
        a * (((1.0 - fraction) * angle).sin() * sin) + b * ((fraction * angle).sin() * sin)
    }
}
///normalized linear interpolation
pub fn nlerp(a: Vec3A, b: Vec3A) -> impl Fn(f32) -> Vec3A {
    move |fraction: f32| -> Vec3A { (a + fraction * (b - a)).normalize() }
}
pub fn lerp(a: Vec3A, b: Vec3A) -> impl Fn(f32) -> Vec3A {
    move |fraction: f32| -> Vec3A { a + fraction * (b - a) }
}

pub fn cantor_pairing(k1: usize, k2: usize) -> usize { (k1 + k2) * (k1 + k2 + 1)/2 + k2 }

///Iteratively subdivide triangle mesh
pub fn subdivide<T: GeometryTopology, Lerp: Fn(f32) -> Vec3A, Interpolator: Fn(Vec3A, Vec3A) -> Lerp>(
    mesh: &T, subdivisions: usize, interpolator: Interpolator,
) -> MeshGeometry {
    let initial_vertices = mesh.vertices();
    let initial_indices = mesh.indices();
    let mut vertices = initial_vertices.to_vec();
    let mut indices = Vec::new();

    struct SubdividedEdgeIterator {
        start: usize,
        end: usize,
        offset: usize,
        length: usize
    }
    impl SubdividedEdgeIterator {
        fn index(&self, index: usize) -> usize {
            if index <= 0 {
                self.start
            }else if index >= self.length - 1 {
                self.end
            }else if self.start < self.end {
                self.offset + index - 1
            }else{
                self.offset + self.length - 2 - index
            }
        }
    }

    fn subdivide_edge<Lerp: Fn(f32) -> Vec3A, Interpolator: Fn(Vec3A, Vec3A) -> Lerp>(
        start: usize, end: usize, subdivisions: usize,
        vertices: &mut Vec<Vec3A>,
        edge_map: Option<&mut HashMap<usize, usize, BuildHasherDefault<PassThroughHasher>>>,
        interpolator: Interpolator,
    ) -> SubdividedEdgeIterator {
        if subdivisions < 1 {
            return SubdividedEdgeIterator{start,end,offset:0,length:2}
        }
        let (min, max) = if start < end {
            (start, end)
        }else{
            (end, start)
        };
        let edge_index = cantor_pairing(min, max);
        let vertex_offset = vertices.len();
        let offset = if let Some(map) = edge_map {
            *map.entry(edge_index).or_insert(vertex_offset)
        }else{
            vertex_offset
        };
        if offset == vertex_offset {
            let interpolate = (interpolator)(vertices[min], vertices[max]);
            let segments = subdivisions + 1;
            vertices.reserve_exact(subdivisions);
            for i in 1..segments {
                vertices.push(interpolate((i as f32) / (segments as f32)));
            }
        }
        SubdividedEdgeIterator{start, end, offset, length: subdivisions + 2}
    }


    let mut edge_map: HashMap<usize, usize, BuildHasherDefault<PassThroughHasher>> = HashMap::default();
    for face in initial_indices.chunks_exact(3){
        let (i0, i1, i2) = (face[0], face[1], face[2]);

        let left = subdivide_edge(i1, i0, subdivisions, &mut vertices, Some(&mut edge_map), &interpolator);
        let right = subdivide_edge(i2, i0, subdivisions, &mut vertices, Some(&mut edge_map), &interpolator);
        let mut bottom = subdivide_edge(i1, i2, subdivisions, &mut vertices, Some(&mut edge_map), &interpolator);
        
        for i in 1..subdivisions+1 {
            let edge_subdivisions = subdivisions - i;
            let top = subdivide_edge(left.index(i), right.index(i), edge_subdivisions, &mut vertices, None, &interpolator);
            
            for j in 0..edge_subdivisions+2 {
                indices.extend_from_slice(&[top.index(j), bottom.index(j), bottom.index(j+1)]);
            }
            for j in 0..edge_subdivisions+1 {
                indices.extend_from_slice(&[bottom.index(j + 1), top.index(j + 1), top.index(j)]);
            }
            bottom = top;
        }
        indices.extend_from_slice(&[i0, bottom.index(0), bottom.index(1)])
    }
    MeshGeometry{vertices,indices}
}

pub fn subdivide_sqrt3<T: GeometryTopology, Lerp: Fn(f32) -> Vec3A, Interpolator: Fn(Vec3A, Vec3A) -> Lerp>(
    mesh: &T, subdivisions: usize, interpolator: Interpolator
) -> MeshGeometry {
    let original_indices = mesh.indices();
    let mut vertices = mesh.vertices().to_vec();
    let mut indices: Vec<usize> = Vec::new();

    let mut edge_map: HashMap<usize, usize, BuildHasherDefault<PassThroughHasher>> = HashMap::default();
    for triangle in original_indices.chunks_exact(3){
        let (i0, i1, i2) = (triangle[0], triangle[1], triangle[2]);
        let (v0, v1, v2) = (vertices[i0], vertices[i1], vertices[i2]);
        let center = interpolator(interpolator(v0, v1)(0.5), v2)(2.0/3.0);
        let (vertex_offset, index_offset) = (vertices.len(), indices.len());
        vertices.push(center);
        indices.extend_from_slice(&[i0,i1,vertex_offset,i1,i2,vertex_offset,i2,i0,vertex_offset]);
        for i in 0..3 {
            let (start, end) = (triangle[i], triangle[(i+1)%3]);
            let edge_offset = index_offset + i * 3;
            let edge_index = if start < end { cantor_pairing(start, end) }else{ cantor_pairing(end, start) };
            if let Some(&opposite_offset) = edge_map.get(&edge_index){
                indices[edge_offset] = indices[opposite_offset + 2];
                indices[opposite_offset] = indices[edge_offset + 2];
            }else{
                edge_map.insert(edge_index, edge_offset);
            }
        }
    }

    MeshGeometry{vertices, indices}
}

#[cfg(test)]
mod tests {
    use bevy::math::Vec3A;
    use super::*;
    fn build_quad() -> MeshGeometry {MeshGeometry{
        vertices: vec![
            Vec3A::new(0.0,0.0,0.0), Vec3A::new(1.0,0.0,0.0), Vec3A::new(1.0,1.0,0.0), Vec3A::new(0.0,1.0,0.0)
        ],
        indices: vec![0,1,2, 0,2,3]
    }}
    #[test]
    fn subdivide_quad(){
        let quad_mesh = build_quad();

        for subdivisions in 1..5 {
            let subdivided = subdivide(&quad_mesh, subdivisions, lerp);
            assert_eq!(subdivided.vertices.len(), (2+subdivisions) * (2+subdivisions));
            assert_eq!(subdivided.indices.len(), (1+subdivisions) * (1+subdivisions) * 2 * 3);
        }
    }
    #[test]
    fn subdivide_sqrt3_quad(){
        let quad_mesh = build_quad();

        let (mut vertex_count, mut triangle_count) = (4, 2);
        for subdivisions in 1..2 {
            let subdivided = subdivide_sqrt3(&quad_mesh, subdivisions, lerp);
            vertex_count = vertex_count + triangle_count;
            triangle_count = triangle_count * 3;
            assert_eq!(subdivided.vertices.len(), vertex_count);
            assert_eq!(subdivided.indices.len(), triangle_count * 3);
        }
    }
}