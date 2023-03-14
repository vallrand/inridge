use std::collections::{VecDeque,HashSet};
use std::hash::Hash;

pub fn breadth_first_search<'a, N: Eq + Hash + Copy, IN: Iterator<Item = &'a N>, C: Ord + Copy>(
    start: &'a N,
    mut adjacency: impl FnMut(&N) -> IN,
    mut heuristic: impl FnMut(&N) -> Option<C>
) -> Vec<N> {
    let Some(mut min_distance) = heuristic(&start) else { return vec![*start] };
    let mut closest_index: usize = 0;
    let mut index: usize = 0;
    let mut stack: Vec<(N, usize, usize)> = vec![(*start, 0, 0)];
    let mut visited: HashSet<N> = HashSet::new();
    visited.insert(*start);
    'outer: while let Some(&(node,depth,_)) = stack.get(index) {
        for adjacent in adjacency(&node) {
            if visited.contains(&adjacent) { continue; }
            stack.push((*adjacent, depth + 1, index));

            let Some(distance) = heuristic(&adjacent) else {
                closest_index = stack.len() - 1;
                break 'outer
            };
            if distance < min_distance {
                closest_index = stack.len() - 1;
                min_distance = distance;
            }
            visited.insert(*adjacent);
        }
        index += 1;
    }
    let mut out: VecDeque<N> = VecDeque::with_capacity(stack[closest_index].1 + 1);
    loop {
        let (node,_depth,parent) = stack[closest_index];
        out.push_front(node);
        closest_index = if parent != closest_index { parent }else{ break Vec::from(out) };
    }
}