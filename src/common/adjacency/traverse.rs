use std::collections::{HashSet, VecDeque};
use std::hash::Hash;
use super::Graph;

pub trait TraversableGraph: Sized {
    type Index: Hash + Eq + Copy;
    fn adjacent(&self, index: Self::Index) -> &[Self::Index];

    fn iter_breadth_first(&self) -> BreadthFirstIterator<'_, Self, fn(Self::Index) -> bool> {
        fn any<T>(_: T) -> bool { true }
        BreadthFirstIterator {
            graph: &self,
            node_filter: any,
            stack: VecDeque::new(),
            visited: HashSet::new(),
            depth_limit: None,
        }
    }
}

pub struct BreadthFirstIterator<'a, G: 'a + TraversableGraph, F> {
    graph: &'a G,
    node_filter: F,
    stack: VecDeque<(G::Index, usize)>,
    visited: HashSet<G::Index>,
    depth_limit: Option<usize>,
}
impl<'a, G: 'a + TraversableGraph, F: FnMut(G::Index) -> bool> Iterator for BreadthFirstIterator<'a, G, F> {
    type Item = G::Index;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let Some((index, depth)) = self.stack.pop_front() else { break; };
            if self.depth_limit.map_or(false, |depth_limit|depth > depth_limit) { continue; }
            if !(self.node_filter)(index) { continue; }

            for &adjacent in self.graph.adjacent(index).iter() {
                if !self.visited.contains(&adjacent) {
                    self.visited.insert(adjacent);
                    self.stack.push_back((adjacent, depth + 1));
                }
            }
            return Some(index)
        }
        None
    }
}
impl<'a, G: 'a + TraversableGraph, F> BreadthFirstIterator<'a, G, F> {
    pub fn with_origin(mut self, origin_index: G::Index) -> Self {
        self.stack.push_back((origin_index, 0));
        self.visited.insert(origin_index);
        self
    }
    pub fn with_limit(mut self, depth_limit: usize) -> Self {
        self.depth_limit = Some(depth_limit);
        self
    }
    pub fn with_filter<NF: FnMut(G::Index) -> bool>(self, filter: NF) -> BreadthFirstIterator<'a, G, NF> {
        BreadthFirstIterator {
            node_filter: filter,
            graph: self.graph, depth_limit: self.depth_limit,
            stack: self.stack, visited: self.visited,
        }
    }
}

impl TraversableGraph for Graph {
    type Index = usize;
    fn adjacent(&self, index: Self::Index) -> &[Self::Index] { self.neighbors(index).unwrap() }
}