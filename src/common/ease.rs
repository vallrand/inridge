use std::ops::*;

pub fn lerp<T, F>(a: T, b: T, f: F) -> T
where T: Copy + Add<T, Output=T> + Sub<T, Output = T> + Mul<F, Output=T> {
    a + (b-a) * f
}
pub fn cubic_bezier<T>(c0: T, c1: T, c2: T, c3: T, t: f32) -> T
where T: Mul<f32, Output = T> + Mul<Output = T> + Add<Output = T> {
    let t2 = t * t; let it = 1.0 - t; let it2 = it * it;
    c0 * it2 * it +
    c1 * 3.0 * it2 * t +
    c2 * 3.0 * it * t2 +
    c3 * t2 * t
}
pub fn cubic_bezier_derivative<T>(c0: T, c1: T, c2: T, c3: T, t: f32) -> T
where T: Mul<f32, Output = T> + Mul<Output = T> + Add<Output = T> {
    let t2 = t * t; let it = 1.0 - t; let it2 = it * it;
    c0 * -it2 +
    c1 * (3.0 * it2 - 2.0 * it) +
    c2 * (-3.0 * t2 + 2.0 * t) +
    c3 * t2
}