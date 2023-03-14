use super::{Overlap, Contains, aabb::AABB};

const INVALID_INDEX: usize = std::usize::MAX;

#[derive(Clone, Copy)]
enum BVHTreeNodeType<T: Sized> {
    Leaf(T),
    Subtree(usize)
}

#[derive(Clone, Copy)]
struct BVHTreeNode<T> {
    parent: usize,
    prev_index: usize,
    next_index: usize,
    node_type: BVHTreeNodeType<T>,
    aabb: AABB,
    weight: i32
}

#[derive(Clone)]
pub struct BVHTree<T: Sized> {
    root: usize,
    pool: Vec<usize>,
    nodes: Vec<BVHTreeNode<T>>,
}

impl<T: PartialEq> Default for BVHTree<T> { fn default() -> Self { Self::with_capacity(0) } }
impl<T: PartialEq> BVHTree<T> {
    #[inline] pub fn with_capacity(capacity: usize) -> Self {
        Self { root: INVALID_INDEX, pool: Vec::default(), nodes: Vec::with_capacity(capacity) }
    }
    #[inline] pub fn is_empty(&self) -> bool { self.root == INVALID_INDEX }
    #[inline] pub fn clear(&mut self){
        self.root = INVALID_INDEX;
        self.pool.clear();
        self.nodes.clear();
    }
    pub fn insert(&mut self, key: T, mut aabb: AABB, padding: f32) -> usize {
        aabb += padding;
        let node_index = self.allocate_node(BVHTreeNode {
            parent: INVALID_INDEX, prev_index: INVALID_INDEX, next_index: INVALID_INDEX,
            node_type: BVHTreeNodeType::Leaf(key), aabb, weight: 1
        });
        if self.root == INVALID_INDEX {
            self.root = node_index;
        } else {
            let mut sibling_index = self.root;
            loop {
                let next_index = self.descend_heuristic(sibling_index, aabb);
                sibling_index = if next_index != sibling_index { next_index }else{ break; };
            }
            let parent_index = self.extend_node(sibling_index);
            self.insert_node_at(node_index, sibling_index);
            self.rebalance(parent_index);
        }
        node_index
    }
    pub fn update(&mut self, node_index: usize, aabb: AABB, padding: f32) -> usize {
        if !self.nodes[node_index].aabb.contains(&aabb, f32::EPSILON) {
            let key: T = self.remove(node_index).unwrap();
            self.insert(key, aabb, padding)
        } else {
            node_index
        }
    }
    pub fn remove(&mut self, node_index: usize) -> Option<T> {
        let BVHTreeNode { parent, next_index, prev_index, .. } = self.nodes[node_index];
        match self.nodes[node_index].node_type {
            BVHTreeNodeType::Leaf(_) => {
                self.replace_node(node_index, next_index, prev_index);
            },
            BVHTreeNodeType::Subtree(first_index) => {
                let mut last_index = first_index;
                loop {
                    let child = &mut self.nodes[last_index];
                    child.parent = parent;
                    last_index = if child.next_index != INVALID_INDEX { child.next_index }else{ break; };
                }
                self.nodes[first_index].prev_index = prev_index;
                self.nodes[last_index].next_index = next_index;
                self.replace_node(node_index, first_index, last_index);
            }
        }
        if parent != INVALID_INDEX {
            if
                if let BVHTreeNodeType::Subtree(first_index) = self.nodes[parent].node_type {
                    self.nodes[first_index].next_index == INVALID_INDEX
                } else { false }
            {
                self.remove(parent);
            } else {
                self.recalculate_node(parent);
                self.rebalance(parent);
            }
        }
        self.pool.push(node_index);
        if let BVHTreeNodeType::Leaf(key) = std::mem::replace(&mut self.nodes[node_index].node_type, BVHTreeNodeType::Subtree(0)) {
            Some(key)
        } else {
            None
        }
    }
    #[inline] fn allocate_node(&mut self, node: BVHTreeNode<T>) -> usize {
        if let Some(node_index) = self.pool.pop() {
            self.nodes[node_index] = node;
            node_index
        } else {
            self.nodes.push(node);
            self.nodes.len() - 1
        }
    }
    #[inline] fn replace_node(&mut self, node_index: usize, first_index: usize, last_index: usize){
        let BVHTreeNode { parent, next_index, prev_index, .. } = self.nodes[node_index];
        if self.root == node_index { self.root = first_index; }

        if next_index != INVALID_INDEX {
            self.nodes[next_index].prev_index = last_index;
        }
        if prev_index != INVALID_INDEX {
            self.nodes[prev_index].next_index = first_index;
        } else if parent != INVALID_INDEX {
            self.nodes[parent].node_type = BVHTreeNodeType::Subtree(first_index);
        }
    }
    fn extend_node(&mut self, node_index: usize) -> usize {
        let BVHTreeNode { parent, next_index, prev_index, aabb, weight, .. } = self.nodes[node_index];
        let parent_index = self.allocate_node(BVHTreeNode {
            parent, prev_index, next_index, aabb, weight,
            node_type: BVHTreeNodeType::Subtree(node_index)
        });
        self.replace_node(node_index, parent_index, parent_index);

        self.nodes[node_index].parent = parent_index;
        self.nodes[node_index].prev_index = INVALID_INDEX;
        self.nodes[node_index].next_index = INVALID_INDEX;

        parent_index
    }
    fn insert_node_at(&mut self, node_index: usize, sibling_index: usize){
        let BVHTreeNode { parent, next_index, .. } = self.nodes[sibling_index];
        self.nodes[node_index].parent = parent;
        self.nodes[node_index].prev_index = sibling_index;
        self.nodes[node_index].next_index = next_index;

        self.nodes[sibling_index].next_index = node_index;
        if next_index != INVALID_INDEX {
            self.nodes[next_index].prev_index = node_index;
        }
        if parent != INVALID_INDEX {
            let BVHTreeNode { aabb, weight, .. } = self.nodes[node_index];
            self.nodes[parent].aabb += aabb;
            self.nodes[parent].weight += weight;
        }
    }
    fn recalculate_node(&mut self, node_index: usize){
        if let BVHTreeNodeType::Subtree(mut child_index) = self.nodes[node_index].node_type {
            let mut aabb: AABB = AABB::default();
            let mut weight: i32 = 0;
            while child_index != INVALID_INDEX {
                let child = &self.nodes[child_index];
                aabb += child.aabb;
                weight += child.weight;
                child_index = child.next_index;
            }
            self.nodes[node_index].aabb = aabb;
            self.nodes[node_index].weight = weight;
        }
    }
    fn rebalance(&mut self, mut node_index: usize){
        loop {
            let max_index: usize = {
                let mut child_index = if let BVHTreeNodeType::Subtree(first_index) = self.nodes[node_index].node_type {
                    first_index
                } else {
                    INVALID_INDEX
                };
                let mut max: i32 = 0;
                let mut max_index: usize = INVALID_INDEX;
                while child_index != INVALID_INDEX {
                    let child = &self.nodes[child_index];
                    if child.weight > max {
                        max = child.weight;
                        max_index = child_index;
                    }
                    child_index = child.next_index;
                }
                if max > 2 * (self.nodes[node_index].weight - max) {
                    max_index
                } else {
                    INVALID_INDEX
                }
            };
            let min_index: usize = if max_index != INVALID_INDEX {
                let mut child_index = if let BVHTreeNodeType::Subtree(first_index) = self.nodes[max_index].node_type {
                    first_index
                } else {
                    INVALID_INDEX
                };
                let mut min: i32 = i32::MAX;
                let mut min_index: usize = INVALID_INDEX;
                while child_index != INVALID_INDEX {
                    let child = &self.nodes[child_index];
                    if child.weight < min {
                        min = child.weight;
                        min_index = child_index;
                    }
                    child_index = child.next_index;
                }
                min_index
            } else { INVALID_INDEX };
            node_index = if max_index != INVALID_INDEX && min_index != INVALID_INDEX {
                self.replace_node(max_index, min_index, min_index);
                self.replace_node(node_index, max_index, max_index);
                self.replace_node(min_index, node_index, node_index);

                let BVHTreeNode { parent, next_index, prev_index, .. } = self.nodes[min_index];
    
                self.nodes[min_index].parent = self.nodes[max_index].parent;
                self.nodes[min_index].next_index = self.nodes[max_index].next_index;
                self.nodes[min_index].prev_index = self.nodes[max_index].prev_index;

                self.nodes[max_index].parent = self.nodes[node_index].parent;
                self.nodes[max_index].next_index = self.nodes[node_index].next_index;
                self.nodes[max_index].prev_index = self.nodes[node_index].prev_index;
    
                self.nodes[node_index].parent = parent;
                self.nodes[node_index].next_index = next_index;
                self.nodes[node_index].prev_index = prev_index;
    
                self.recalculate_node(node_index);
                self.nodes[max_index].aabb = self.nodes[max_index].aabb + self.nodes[node_index].aabb;
                self.nodes[max_index].weight += self.nodes[node_index].weight - self.nodes[min_index].weight;
                self.nodes[max_index].parent
            } else {
                self.nodes[node_index].parent
            };
            if node_index != INVALID_INDEX {
                self.recalculate_node(node_index);
            } else {
                break;
            }
        }
    }
    fn descend_heuristic(&self, parent_index: usize, aabb: AABB) -> usize {
        let parent = &self.nodes[parent_index];
        if let BVHTreeNodeType::Subtree(first_index) = parent.node_type {
            let area = (aabb + parent.aabb).surface_area();
            let inheritance_cost = 2.0 * (area - parent.aabb.surface_area());
    
            let mut min_index = parent_index;
            let mut min_cost = 2.0 * area;
            
            let mut child_index = first_index;
            while child_index != INVALID_INDEX {
                let child = &self.nodes[child_index];
                let descend_cost = inheritance_cost + match child.node_type {
                    BVHTreeNodeType::Leaf(_) => (child.aabb + aabb).surface_area(),
                    BVHTreeNodeType::Subtree(_) => (child.aabb + aabb).surface_area() - child.aabb.surface_area(),
                };
                if min_cost >= descend_cost {
                    min_cost = descend_cost;
                    min_index = child_index;
                }
                child_index = child.next_index;
            }
            min_index
        } else {
            parent_index
        }
    }
    pub fn query<'a, F: Overlap<&'a AABB>>(&'a self, filter: &'a F) -> BVHTreeIterator<'a, T, F> {
        BVHTreeIterator { tree: self, filter, node_index: self.root, node_exit: false }
    }
}

pub struct BVHTreeIterator<'a, T, F: Overlap<&'a AABB>> {
    tree: &'a BVHTree<T>,
    filter: &'a F,
    node_index: usize,
    node_exit: bool
}

impl<'a, T, F: Overlap<&'a AABB>> Iterator for BVHTreeIterator<'a, T, F> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        while self.node_index != INVALID_INDEX {
            let node = &self.tree.nodes[self.node_index];
            if self.node_exit || !self.filter.overlap(&node.aabb, f32::EPSILON) {
                if node.next_index == INVALID_INDEX {
                    self.node_index = node.parent;
                    self.node_exit = true;
                } else {
                    self.node_index = node.next_index;
                    self.node_exit = false;
                }
            } else {
                match node.node_type {
                    BVHTreeNodeType::Leaf(ref key) => {
                        self.node_exit = true;
                        return Some(key);
                    },
                    BVHTreeNodeType::Subtree(first_child) => {
                        self.node_exit = false;
                        self.node_index = first_child;
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use std::ops::AddAssign;
    use bevy::math::Vec3A;
    use super::*;
    fn hash11(u: u32) -> f32 { ((u as f32).sin() * 43758.5453123).fract() }
    fn add_random_objects(tree: &mut BVHTree<u32>, list: &mut Vec<(usize,u32,AABB)>, amount: usize, random: impl Fn() -> f32){
        for i in 0..amount {
            let aabb = AABB::from(100.0 * (Vec3A::new(random(), random(), random()) * 2.0 - 1.0)) + 5.0;
            let node_index = tree.insert(i as u32, aabb, 0.0);
            list.push((node_index, i as u32, aabb));
        }
    }
    fn remove_random_objects(tree: &mut BVHTree<u32>, list: &mut Vec<(usize,u32,AABB)>, amount: usize, random: impl Fn() -> f32){
        for _ in 0..amount.min(list.len()) {
            let index = (random() * list.len() as f32) as usize;
            let (node_index, _, _) = list.swap_remove(index);
            tree.remove(node_index);
        }
    }
    struct AABBProxy {
        aabb: AABB,
        visited: std::cell::RefCell<usize>
    }
    impl Overlap<&AABB> for AABBProxy {
        fn overlap(&self, rhs: &AABB, epsilon: f32) -> bool {
            self.visited.borrow_mut().add_assign(1);
            self.aabb.overlap(rhs, epsilon)
        }
    }
    fn test_aabb_query(tree: &BVHTree<u32>, list: &Vec<(usize,u32,AABB)>, amount: usize, random: impl Fn() -> f32) -> usize {
        verify_tree(tree, tree.root);
        let mut proxy = AABBProxy { aabb: AABB::default(), visited: std::cell::RefCell::new(0) };
        for _ in 0..amount {
            let a = 100.0 * (Vec3A::new(random(), random(), random()) * 2.0 - 1.0);
            let b = 100.0 * (Vec3A::new(random(), random(), random()) * 2.0 - 1.0);
            proxy.aabb = AABB::default() + a + b;
            let mut expected: Vec<u32> = list.clone().into_iter()
            .filter(|(_, _, aabb)|aabb.overlap(&proxy.aabb, f32::EPSILON))
            .map(|(_, key,_)|key)
            .collect();
            for &key in tree.query(&proxy) {
                let index = expected.iter().position(|&other|other==key);
                assert_ne!(index, None);
                expected.swap_remove(index.unwrap());
            }
            assert_eq!(expected.len(), 0);
        }
        let total = *proxy.visited.borrow();
        total
    }
    fn verify_tree(tree: &BVHTree<u32>, node_index: usize){
        let node = &tree.nodes[node_index];
        if let BVHTreeNodeType::Subtree(mut child_index) = node.node_type {
            let (mut aabb, mut weight) = (AABB::default(), 0);
            while child_index != INVALID_INDEX {
                verify_tree(tree, child_index);
                aabb += tree.nodes[child_index].aabb;
                weight += tree.nodes[child_index].weight;
                child_index = tree.nodes[child_index].next_index;
            }
            assert_eq!(weight, node.weight);
            assert!(node.aabb.relative_eq(&aabb, f32::EPSILON));
        }
    }

    #[test] pub fn query_aabb_points(){
        let mut tree: BVHTree<u32> = BVHTree::with_capacity(0);
        let mut list: Vec<(usize,u32,AABB)> = Vec::new();
        let seed: std::cell::RefCell<u32> = std::cell::RefCell::new(1);
        let random = || -> f32 { let next = 1 + *seed.borrow(); *seed.borrow_mut() = next; hash11(next) };

        add_random_objects(&mut tree, &mut list, 16, random);
        test_aabb_query(&tree, &list, 64, random);
        add_random_objects(&mut tree, &mut list, 32, random);
        test_aabb_query(&tree, &list, 32, random);
        remove_random_objects(&mut tree, &mut list,24, random);
        test_aabb_query(&tree, &list, 16, random);
        add_random_objects(&mut tree, &mut list, 128, random);
        remove_random_objects(&mut tree, &mut list,8, random);
        test_aabb_query(&tree, &list, 64, random);

        remove_random_objects(&mut tree, &mut list,24+128-8, random);
        assert_eq!(tree.is_empty(), true);
    }
}