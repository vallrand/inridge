use std::ops::*;
use std::f32::consts::FRAC_PI_2;

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

#[derive(serde::Deserialize, Clone, Copy, Default, PartialEq, Debug)]
pub enum Smoothing {
    #[default] None,
    Exponential(f32)
}
impl Smoothing {
    pub fn calculate(&self, delta_time: f32) -> f32 { match self {
        Smoothing::None => 1.0,
        Smoothing::Exponential(smoothness) => 1.0 - f32::powf(1.0 - smoothness, delta_time)
    } }
}

#[derive(serde::Deserialize, Clone, Copy, PartialEq, Debug)]
pub enum SimpleCurve {
    Power(i32),
    Offset(f32),
    Sine,
    Circular,
}
impl SimpleCurve {
    #[inline] pub fn calculate(&self, x: f32) -> f32 {
        match *self {
            Self::Power(power) => x.powi(power),
            Self::Offset(offset) => 1.0 - (1.0 + offset) * (1.0 - x) / (1.0 + offset - x),
            Self::Sine => 1.0 - (x * FRAC_PI_2).cos(),
            Self::Circular => 1.0 - (1.0 - x * x).sqrt(),
        }
    }
}

#[derive(serde::Deserialize, Clone, Copy, PartialEq, Default, Debug)]
pub enum Ease {
    #[default] Linear,
    Hold(f32),
    Step(f32),
    In(SimpleCurve),
    Out(SimpleCurve),
    InOut(SimpleCurve),
    Bezier(BezierCurve)
}
impl Ease {
    #[inline] pub fn calculate(&self, x: f32) -> f32 {
        match *self {
            Ease::Linear => x,
            Ease::Hold(y) => y,
            Ease::Step(step) => if x < step { 0.0 }else{ 1.0 },
            Ease::In(curve) => curve.calculate(x),
            Ease::Out(curve) => 1.0 - curve.calculate(1.0 - x),
            Ease::InOut(curve) => if x < 0.5 {
                0.5 * curve.calculate(2.0 * x)
            }else{
                1.0 - 0.5 * curve.calculate(2.0 - 2.0 * x)
            },
            Ease::Bezier(curve) => curve.calculate(x)
        }
    }
}

#[derive(serde::Deserialize, Copy, Clone, PartialEq, Debug)]
pub struct BezierCurve {
    ax: f32, bx: f32, cx: f32,
    ay: f32, by: f32, cy: f32,
}
impl BezierCurve {
    const NEWTON_ITERATIONS: usize = 4;
    const BINARY_ITERATIONS: usize = 8;
    const PRECISION: f32 = f32::EPSILON;
    pub fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        let cx = 3.0 * x1;
        let cy = 3.0 * y1;
        let bx = 3.0 * x2 - 2.0 * cx;
        let by = 3.0 * y2 - 2.0 * cy;
        let ax = 1.0 - bx - cx;
        let ay = 1.0 - by - cy;
        Self { ax, ay, bx, by, cx, cy }
    }
    #[inline] pub fn calculate(&self, x: f32) -> f32 {
        if x <= 0.0 { 0.0 }else if x >= 1.0 { 1.0 }else{ self.yt(self.tx(x, Self::PRECISION)) }
    }
    #[inline] fn dxdt(&self, t: f32) -> f32 { 3.0 * self.ax * t * t + 2.0 * self.bx * t + self.cx }
    #[inline] fn xt(&self, t: f32) -> f32 { ((self.ax * t + self.bx) * t + self.cx) * t }
    #[inline] fn yt(&self, t: f32) -> f32 { ((self.ay * t + self.by) * t + self.cy) * t }
    fn tx(&self, x: f32, epsilon: f32) -> f32 {
        let mut t_min: f32 = 0.0;
        let mut t_max: f32 = 1.0;
        let mut t: f32 = x;
        for _ in 0..Self::NEWTON_ITERATIONS {
            let delta = self.xt(t) - x;
            if delta < -epsilon {
                t_min = t;
            } else if delta > epsilon {
                t_max = t;
            } else {
                return t;
            }
            let slope = self.dxdt(t);
            if slope.abs() < epsilon { return t; }
            t -= delta / slope;
        }
        for _ in 0..Self::BINARY_ITERATIONS {
            let delta = self.xt(t) - x;
            if delta < -epsilon {
                t_min = t;
            } else if delta > epsilon {
                t_max = t;
            } else {
                return t;
            }
            t = (t_min + t_max) / 2.0;
        }
        t
    }
}