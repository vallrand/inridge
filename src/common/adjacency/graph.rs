#[derive(Clone, Default)]
pub struct Graph {
    rank: usize,
    pointers: Vec<(usize, usize)>,
    adjacency: Vec<usize>,
}

impl Graph {
    pub fn len(&self) -> usize { self.adjacency.len() }
    pub fn edge_key(&self, start: usize, end: usize) -> Option<usize> {
        let (min, max) = if start < end { (start, end) }else{ (end, start) };
        let adjacent = self.neighbors(min)?;
        adjacent.iter()
        .position(|pair| pair == &max)
        .map(|i| min * self.rank + i)
    }
    pub fn neighbors(&self, index: usize) -> Option<&[usize]> {
        self.pointers.get(index).map(|(offset, length)| {
            &self.adjacency[*offset..(*offset+*length)]
        })
    }
    pub fn contains(&self, start: usize, end: usize) -> bool {
        self.neighbors(start).map_or(false,|list|list.contains(&end))
    }
}

use super::super::geometry::GeometryTopology;
impl<T: GeometryTopology> From<&T> for Graph {
    fn from(mesh: &T) -> Self {
        let indices = mesh.indices();
        let vertex_count: usize = mesh.vertices().len();
        let mut jagged: Vec<Vec<usize>> = (0..vertex_count).map(|_| Vec::new()).collect();
        for face in indices.chunks_exact(3){
            jagged[face[0]].extend_from_slice(&[face[1],face[2]]);
            jagged[face[1]].extend_from_slice(&[face[2],face[0]]);
            jagged[face[2]].extend_from_slice(&[face[0],face[1]]);
        }

        let mut max_rank: usize = 0;
        let mut adjacency: Vec<usize> = Vec::new();
        let pointers: Vec<(usize, usize)> = jagged.into_iter().map(|mut edges|{
            let offset = adjacency.len();

            let pairs = edges.len() >> 1;
            for i in 1..pairs {
                let prev = edges[(i-1 << 1) + 1];
                let next = (i..pairs).find(|&j| edges[j<<1] == prev).unwrap_or(i);
                if next != i {
                    edges.swap(i << 1, next << 1);
                    edges.swap((i << 1) + 1, (next << 1) + 1);
                }
            }
            for index in edges.into_iter() {
                if !adjacency[offset..].contains(&index) {
                    adjacency.push(index);
                }
            }
            max_rank = max_rank.max(adjacency.len() - offset);
            (offset,adjacency.len() - offset)
        }).collect();

        Self{ adjacency, pointers, rank: max_rank }
    }
}