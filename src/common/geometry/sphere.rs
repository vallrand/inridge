use bevy::math::Vec3A;
use std::f32::consts::{PI,TAU,SQRT_2};

#[derive(Clone, Copy)]
pub struct Sphere {
    pub origin: Vec3A,
    pub radius: f32
}
impl Default for Sphere { fn default() -> Self { Self { origin: Vec3A::ZERO, radius: 1.0 } } }

pub fn fibonacci_lattice(limit: usize) -> Vec<[f32; 3]> {
    let mut list: Vec<[f32; 3]> = Vec::with_capacity(limit);
    const GOLDEN_ANGLE: f32 = PI * (3.0 - 1.0 / SQRT_2);
    for i in 0..limit {
        let y: f32 = 1.0 - 2.0 * i as f32 / (limit - 1) as f32;
        let radius: f32 = (1.0 - y * y).sqrt();
        let theta: f32 = GOLDEN_ANGLE * i as f32;
        list.push([theta.cos() * radius, y, theta.sin() * radius])
    }
    list
}

///https://en.wikipedia.org/wiki/Unit_sphere
fn unit_sphere_area(mut dimensions: usize) -> f32 {
    let mut area: f32 = 1.0;
    while dimensions > 2 {
        area *= TAU / (dimensions - 2) as f32;
        dimensions -= 2;
    }
    area * if dimensions == 2 { TAU } else { 2.0 }
}

///https://www.cmu.edu/biolphys/deserno/pdf/sphere_equi.pdf
pub fn equidistant_distribution<const D: usize>(density: usize) -> Vec<[f32; D]> {
    let mut list: Vec<[f32; D]> = Vec::with_capacity(density);
    let area: f32 = unit_sphere_area(D) / density as f32;

    #[derive(Clone, Copy)]
    struct Layer {
        fraction: f32,
        total: f32,
        area: f32,
        segments: usize,
        index: usize,
    }
    fn recalculate_layer<const D: usize>(dimension: usize, fraction: f32, area: f32) -> Layer {
        let delta: f32 = area.powf(1.0 / (D - 1 - dimension) as f32);
        let total: f32 = if dimension >= D - 2 { TAU }else{ PI };
        let segments: usize = (fraction * total / delta).round() as usize;
        let reduced_area: f32 = area / (fraction * total / segments as f32);
        Layer { fraction, total, area: reduced_area, segments, index: 0 }
    }
    let mut stack: [Layer; D] = [recalculate_layer::<D>(0, 1.0, area); D];
    let mut vector: [f32; D] = [0.0; D];
    let mut dimension: usize = 0;
    loop {
        let Layer { fraction, total, area, index, segments } = stack[dimension];
        if index >= segments && dimension > 0 {
            dimension -= 1;
            continue;
        }else if index >= segments {
            break;
        }
        let angle: f32 = total * (index as f32 + 0.5) / segments as f32;
        stack[dimension].index += 1;
        vector[dimension] = fraction * angle.cos();
        let reduced_fraction = fraction * angle.sin();
        if dimension + 1 >= D - 1 {
            vector[dimension + 1] = reduced_fraction;
            list.push(vector.clone())
        }else{
            dimension += 1;
            stack[dimension] = recalculate_layer::<D>(dimension, reduced_fraction, area)
        }
    }
    list
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;
    #[test] fn sphere_area(){
        const ERROR_MARGIN: f32 = 100.0 * f32::EPSILON;
        for (dimension, expected) in vec![
            2.0,
            PI * 2.0,
            PI * 4.0,
            PI * PI * 2.0,
            PI * PI * 8.0 / 3.0,
            PI * PI * PI,
            PI * PI * PI * 16.0 / 15.0,
            PI * PI * PI * PI * 1.0 / 3.0,
            PI * PI * PI * PI * 32.0 / 105.0,
            PI * PI * PI * PI * PI * 1.0 / 12.0,
        ].into_iter().enumerate() {
            let area = unit_sphere_area(dimension + 1);
            assert!((area - expected).abs() < ERROR_MARGIN, "\nexpected {} got {}", expected, area);
        }
    }
    #[test] fn sphere_distributions(){
        let list_2d = equidistant_distribution::<2>(16);
        let list_3d = equidistant_distribution::<3>(32);
        let list_4d = equidistant_distribution::<4>(52);

        let margin_2d = list_2d.iter().map(|v0|
            list_2d.iter().filter(|v1|v0!=*v1).fold(f32::MAX, |min, v1|
                min.min(f32::hypot(v0[0]-v1[0], v0[1]-v1[1]))
            )
        ).fold((f32::MAX,f32::MIN), |(min, max), distance|
            (min.min(distance), max.max(distance))
        );

        let margin_3d = list_3d.iter().map(|v0|
            list_3d.iter().filter(|v1|v0!=*v1).fold(f32::MAX, |min, v1|
                min.min((
                    (v0[0]-v1[0]).powi(2) + (v0[1]-v1[1]).powi(2) + (v0[2]-v1[2]).powi(2)
                ).sqrt())
            )
        ).fold((f32::MAX,f32::MIN), |(min, max), distance|
            (min.min(distance), max.max(distance))
        );

        let margin_4d = list_4d.iter().map(|v0|
            list_4d.iter().filter(|v1|v0!=*v1).fold(f32::MAX, |min, v1|
                min.min((
                    (v0[0]-v1[0]).powi(2) + (v0[1]-v1[1]).powi(2) + (v0[2]-v1[2]).powi(2) + (v0[3]-v1[3]).powi(2)
                ).sqrt())
            )
        ).fold((f32::MAX,f32::MIN), |(min, max), distance|
            (min.min(distance), max.max(distance))
        );

        assert_eq!(list_2d.len(), 16);
        assert_eq!(list_3d.len(), 32);
        assert_eq!(list_4d.len(), 52);
        assert!((margin_2d.0 - margin_2d.1).abs() < f32::EPSILON * 1e2, "2d {:?}", margin_2d);
        assert!((margin_3d.0 - margin_3d.1).abs() < f32::EPSILON * 1e6, "3d {:?}", margin_3d);
        assert!((margin_4d.0 - margin_4d.1).abs() < f32::EPSILON * 1e6, "4d {:?}", margin_4d);        
    }
}