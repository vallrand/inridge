trait VecNDExt<const D: usize> where Self: Sized {
    fn dot(&self, rhs: &Self) -> f32;
    fn from_sub(&mut self, lhs: &Self, rhs: &Self);
    fn distance_squared(&self, rhs: &Self) -> f32;
    fn find_farthest(&self, vertices: &[Self]) -> (usize, f32);
}
impl<const D: usize> VecNDExt<D> for [f32; D] {
    #[inline] fn dot(&self, rhs: &Self) -> f32 {
        let mut dot: f32 = 0.0;
        for d in 0..D { dot += self[d] * rhs[d]; }
        dot
    }
    #[inline] fn from_sub(&mut self, lhs: &Self, rhs: &Self) {
        for d in 0..D { self[d] = lhs[d] - rhs[d]; }
    }
    #[inline] fn distance_squared(&self, rhs: &Self) -> f32 {
        let mut distance: f32 = 0.0;
        for d in 0..D {
            let delta = self[d] - rhs[d];
            distance += delta * delta;
        }
        distance
    }
    #[inline] fn find_farthest(&self, vertices: &[Self]) -> (usize, f32) {
        let mut max_distance_squared: f32 = 0.0;
        let mut farthest_index: usize = 0;
        for i in 0..vertices.len() {
            let distance = VecNDExt::distance_squared(&vertices[i], &self);
            if distance >= max_distance_squared {
                max_distance_squared = distance;
                farthest_index = i;
            }
        }
        (farthest_index, max_distance_squared)
    }
}

pub fn bounding_sphere_aabb<const D: usize>(vertices: &[[f32; D]], _epsilon: f32) -> ([f32; D], f32) {
    let mut min: [f32; D] = [f32::MAX; D];
    let mut max: [f32; D] = [f32::MIN; D];
    for vertex in vertices.iter() {
        for d in 0..D {
            min[d] = min[d].min(vertex[d]);
            max[d] = max[d].max(vertex[d]);
        }
    }
    let mut center: [f32; D] = [0.0; D];
    let mut radius_squared: f32 = 0.0;
    for d in 0..D { center[d] = (min[d] + max[d]) / 2.0; }
    for vertex in vertices.iter() {
        radius_squared = radius_squared.max(VecNDExt::distance_squared(&center, vertex));
    }
    (center, radius_squared.sqrt())
}

///https://github.com/hbf/miniball
pub fn bounding_sphere_fischer<const D: usize>(vertices: &[[f32; D]], epsilon: f32) -> ([f32; D], f32) {
    assert!(vertices.len() > 0, "len > 0");
    let mut center: [f32; D] = vertices[vertices.len() - 1];
    let (farthest, mut radius_squared) = VecNDExt::find_farthest(&center, &vertices[..vertices.len() - 1]);
    let supports: &mut [usize] = &mut vec![farthest; D + 1];
    let mut rank: usize = 0;
    let mut q: [[f32; D]; D] = [[0.0; D]; D];
    let mut r: [[f32; D]; D] = [[0.0; D]; D];
    for d in 0..D { q[d][d] = 1.0; }

    ///Givens coefficients satisfying <i>cos * a + sin * b = +/- (a^2 + b^2)</i> and <i>cos * b - sin * a = 0</i>
    #[inline] fn givens_rotation(a: f32, b: f32) -> (f32, f32) {
        if b == 0.0 {
            (0.0, 1.0)
        } else if b.abs() > a.abs() {
            let t = a / b;
            let sin = (1.0 + t * t).sqrt().recip();
            (sin, sin * t)
        } else {
            let t = b / a;
            let cos = (1.0 + t * t).sqrt().recip();
            (cos * t, cos)
        }
    }
    ///hessenberg lower
    #[inline] fn clear_qr_matrix<const D: usize>(r: &mut [[f32; D]; D], q: &mut [[f32; D]; D], rank: usize, index: usize){
        for i in index..rank {
            let (sin, cos) = givens_rotation(r[i][i], r[i][i + 1]);
            r[i][i] = cos * r[i][i] + sin * r[i][i + 1];
            for j in i+1..rank {
                let a = r[j][i];
                let b = r[j][i+1];
                r[j][i] = cos * a + sin * b;
                r[j][i+1] = cos * b - sin * a;
            }
            for j in 0..D {
                let a = q[i][j];
                let b = q[i+1][j];
                q[i][j] = cos * a + sin * b;
                q[i+1][j] = cos * b - sin * a;
            }
        }
    }

    #[inline] fn push_qr_column<const D: usize>(r: &mut [[f32; D]; D], q: &mut [[f32; D]; D], rank: usize, difference: &[f32; D]){
        assert!(rank < D);
        for i in 0..D { r[rank][i] = VecNDExt::dot(&q[i], difference); }
        for i in (rank..D-1).rev() {
            let (sin, cos) = givens_rotation(r[rank][i], r[rank][i+1]);
            r[rank][i] = cos * r[rank][i] + sin * r[rank][i+1];
            for j in 0..D {
                let a = q[i][j];
                let b = q[i+1][j];
                q[i][j] = cos * a + sin * b;
                q[i+1][j] = cos * b - sin * a;
            }
        }
    }
    #[inline] fn pop_qr_column<const D: usize>(r: &mut [[f32; D]; D], q: &mut [[f32; D]; D], rank: usize, difference: &[f32; D]){
        let mut lambdas: [f32; D] = [0.0; D];
        for i in 0..D { lambdas[i] = VecNDExt::dot(&q[i], &difference); }
        for i in (0..D-1).rev() {
            let (sin, cos) = givens_rotation(lambdas[i], lambdas[i+1]);
            lambdas[i] = cos * lambdas[i] + sin * lambdas[i+1];
            r[i][i+1] = -sin * r[i][i];
            r[i][i] = cos * r[i][i];

            for j in i+1..rank {
                let a = r[j][i];
                let b = r[j][i+1];
                r[j][i] = cos * a + sin * b;
                r[j][i+1] = cos * b - sin * a;
            }
            for j in 0..D {
                let a = q[i][j];
                let b = q[i+1][j];
                q[i][j] = cos * a + sin * b;
                q[i+1][j] = cos * b - sin * a;
            }
        }
        for i in 0..rank { r[i][0] += lambdas[0] }
    }

    fn closest_support<const D: usize>(r: &[[f32; D]; D], q: &[[f32; D]; D], rank: usize, direction: &[f32; D]) -> Option<usize> {
        let mut lambdas: [f32; D] = [0.0; D];
        let mut lambda_min: f32 = 1.0;
        let mut index_min: usize = 0;

        let mut origin_lambda = 1.0;
        for i in (0..rank).rev() {
            let mut lambda = VecNDExt::dot(&q[i], &direction);
            for k in i+1..rank { lambda -= lambdas[k] * r[k][i]; }
            lambda /= r[i][i];
            lambdas[i] = lambda;
            origin_lambda -= lambda;
            if lambda < lambda_min {
                lambda_min = lambda;
                index_min = i;
            }
        }
        if origin_lambda < lambda_min {
            lambda_min = origin_lambda;
            index_min = rank;
        }
        if lambda_min <= 0.0 {
            Some(index_min)
        } else {
            None
        }
    }
    
    let mut difference: [f32; D] = [0.0; D];
    let mut shortest_affine_direction: [f32; D] = [0.0; D];
    let mut shortest_affine_distance_squared: f32;
    let mut prev_rank: usize = 1;
    loop {
        shortest_affine_distance_squared = if prev_rank != rank {
            VecNDExt::from_sub(&mut shortest_affine_direction, &vertices[supports[rank]], &center);
            for i in 0..rank {
                let dot = VecNDExt::dot(&shortest_affine_direction, &q[i]);
                for d in 0..D { shortest_affine_direction[d] -= dot * q[i][d]; }
            }
            VecNDExt::dot(&shortest_affine_direction, &shortest_affine_direction)
        } else { 0.0 };
        prev_rank = rank;
        if rank == D || shortest_affine_distance_squared <= epsilon * radius_squared {
            VecNDExt::from_sub(&mut difference, &center, &vertices[supports[rank]]);
            if let Some(index) = closest_support(&r, &q, rank, &difference) {
                assert!(rank > 0); rank -= 1;
                if index == rank + 1 {
                    VecNDExt::from_sub(&mut difference, &vertices[supports[rank + 1]], &vertices[supports[rank]]);
                    pop_qr_column(&mut r, &mut q, rank, &difference);
                    clear_qr_matrix(&mut r, &mut q, rank, 0);
                } else {
                    r[index..].rotate_left(1);
                    supports[index..].rotate_left(1);
                    clear_qr_matrix(&mut r, &mut q, rank, index);
                }
                continue;
            } else {
                break;
            }
        }
        let (fraction, bound_index) = {
            let mut fraction_min: f32 = 1.0;
            let mut index_min: Option<usize> = None;
            for i in 0..vertices.len() {
                if supports[..=rank].contains(&i) { continue; }
                let mut distance_squared: f32 = shortest_affine_distance_squared;
                for d in 0..D {
                    difference[d] = vertices[i][d] - center[d];
                    distance_squared -= shortest_affine_direction[d] * difference[d];
                }
                if distance_squared < epsilon * shortest_affine_distance_squared { continue; }
                let bound = (radius_squared - VecNDExt::dot(&difference, &difference)) / 2.0 / distance_squared;
                if bound > 0.0 && bound < fraction_min {
                    fraction_min = bound;
                    index_min = Some(i);
                }
            }
            (fraction_min, index_min)
        };
        for d in 0..D { center[d] += fraction * shortest_affine_direction[d]; }
        radius_squared = VecNDExt::distance_squared(&center, &vertices[supports[rank]]);

        if let Some(index) = bound_index {
            VecNDExt::from_sub(&mut difference, &vertices[index], &vertices[supports[rank]]);
            push_qr_column(&mut r, &mut q, rank, &difference);
            supports[rank + 1] = supports[rank];
            supports[rank] = index;
            rank += 1;
        }
    }
    (center, radius_squared.sqrt())
}

///https://people.inf.ethz.ch/gaertner/subdir/software/miniball.html
pub fn bounding_sphere_gaertner<const D: usize>(vertices: &[[f32; D]], epsilon: f32) -> ([f32; D], f32) {
    assert!(vertices.len() > 0, "len > 0");
    let mut supports: Vec<usize> = vec![0; D + 1];
    let mut stack: Vec<usize> = vec![0; D + 1];
    let mut support_last: usize = 0;
    
    let mut affine_direction: [[f32; D]; D] = [[0.0; D]; D];
    let mut affine_distance: [f32; D] = [0.0; D];
    let mut difference: [f32; D] = [0.0; D];
    
    let mut prev_center: [[f32; D]; D] = [[0.0; D]; D];
    let mut prev_radius_squared: [f32; D] = [0.0; D];
    let mut center = prev_center[0];
    let mut radius_squared: f32 = -1.0;

    let mut best_radius_squared: f32 = radius_squared;
    let mut limit: usize = 0;
    while limit < 2 {
        let (pivot_index, max_distance_squared) = VecNDExt::find_farthest(&center, &vertices);
        if (radius_squared < 0.0 || max_distance_squared - radius_squared > 0.0) && !supports[..support_last].contains(&pivot_index) {

            center = vertices[pivot_index];
            radius_squared = 0.0;
            prev_center[0] = center;
            prev_radius_squared[0] = radius_squared;

            stack[0] = support_last;
            let mut last_index: usize = 0;
            let mut exit: bool = false;
            'pivot: while !exit || last_index > 0 {
                let start_index = if exit {
                    let i = stack[last_index];
                    last_index -= 1;
                    support_last += 1;
                    supports[..=i].rotate_right(1);
                    i + 1
                } else {
                    support_last = 0;
                    exit = true;
                    if last_index == D { continue 'pivot; }
                    0
                };
                for i in start_index..stack[last_index] {
                    if 0.0 < VecNDExt::distance_squared(&vertices[supports[i]], &center) - radius_squared {
                        VecNDExt::from_sub(&mut difference, &vertices[supports[i]], &vertices[pivot_index]);
                        affine_direction[last_index] = difference;
                        for i in 0..last_index {
                            let offset = VecNDExt::dot(&affine_direction[i], &difference) * 2.0 / affine_distance[i];
                            for d in 0..D { affine_direction[last_index][d] -= offset * affine_direction[i][d]; }
                        }
                        affine_distance[last_index] = 2.0 * VecNDExt::dot(&affine_direction[last_index], &affine_direction[last_index]);
                        if affine_distance[last_index] >= epsilon * epsilon *  radius_squared {
                            let delta = VecNDExt::distance_squared(&vertices[supports[i]], &prev_center[last_index]) - prev_radius_squared[last_index];
                            let fraction = delta / affine_distance[last_index];

                            for d in 0..D { center[d] = prev_center[last_index][d] + fraction * affine_direction[last_index][d]; }
                            radius_squared = prev_radius_squared[last_index] + delta * fraction / 2.0;

                            last_index += 1;
                            if last_index < D {
                                prev_center[last_index] = center;
                                prev_radius_squared[last_index] = radius_squared;
                            }
                            stack[last_index] = i;
                            exit = false;
                            continue 'pivot;
                        }
                    }
                }
            }
            assert_eq!(last_index, 0);
            supports.rotate_right(1);
            supports[0] = pivot_index;
            if support_last < D + 1 { support_last += 1; }
        }
        limit = if best_radius_squared < radius_squared {
            best_radius_squared = radius_squared; 0
        } else { limit + 1 };
    }
    (center, radius_squared.sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn hash11(u: u32) -> f32 { ((u as f32).sin() * 43758.5453123).fract() }

    fn verify_bounding_sphere<const D: usize>((center, radius): &([f32; D], f32), vertices: &[[f32; D]], epsilon: f32){
        for vertex in vertices.iter() {
            let distance = VecNDExt::distance_squared(center, vertex).sqrt();
            assert!(distance - radius < epsilon, "{} <= {}", distance, radius)
        }
    }
    fn generate_random_vertices<const D: usize>(amount: usize, random: impl Fn() -> f32) -> Vec<[f32; D]> {
        (0..amount).map(|_| -> [f32; D] {
            let mut vertex: [f32; D] = [0.0; D];
            for d in 0..D { vertex[d] = (random() * 2.0 - 1.0) * 100.0; }
            vertex
        }).collect()
    }

    #[test] pub fn calc_bounding_sphere(){
        let seed: std::cell::RefCell<u32> = std::cell::RefCell::new(1);
        let random = || -> f32 { let next = 1 + *seed.borrow(); *seed.borrow_mut() = next; hash11(next) };
        const DIMENSIONS: usize = 3;

        for amount in (2..10).chain((1..10).map(|i|i*100)).chain((1..10).map(|i|i*10000)) {
            let vertices = generate_random_vertices::<DIMENSIONS>(amount as usize, random);

            let sphere_fisher = bounding_sphere_fischer::<DIMENSIONS>(&vertices, f32::EPSILON);
            verify_bounding_sphere(&sphere_fisher, &vertices, sphere_fisher.1 * 10.0 * f32::EPSILON);

            let sphere_gaertner = bounding_sphere_gaertner::<DIMENSIONS>(&vertices, f32::EPSILON);
            verify_bounding_sphere(&sphere_gaertner, &vertices, sphere_gaertner.1 * 10.0 * f32::EPSILON);

            assert!((sphere_fisher.1 - sphere_gaertner.1).abs() < sphere_fisher.1.max(sphere_gaertner.1) * 10.0 * f32::EPSILON, "{} = {}", sphere_fisher.1, sphere_gaertner.1);
        }
    }
}