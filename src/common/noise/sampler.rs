use std::marker::PhantomData;

use super::helpers::*;

pub trait HeightSampler<T> {
    fn sample(&self, coordinate: T, seed: u32) -> f32;
}

#[derive(PartialEq)]
pub enum FractalType {
    None,
    Fbm,
    Ridged,
    PingPong(f32)
}
pub struct FractalSampler<T, S: HeightSampler<T>> {
    marker: PhantomData<T>,
    pub sampler: S,
    pub frequency: f32,
    pub fractal_type: FractalType,
    pub octaves: usize,
    pub lacunarity: f32,
    pub strength: f32,
    pub gain: f32
}

impl<T,S: HeightSampler<T>> From<S> for FractalSampler<T,S> {
    fn from(sampler: S) -> Self {Self{
        marker: PhantomData,
        sampler,
        frequency: 0.01,
        fractal_type: FractalType::None,
        octaves: 4,
        lacunarity: 2.0,
        gain: 0.5,
        strength: 0.0,
    }}
}

impl<T, S: HeightSampler<T>> HeightSampler<T> for FractalSampler<T, S>
where T: std::ops::MulAssign<f32> + Copy {
    fn sample(&self, mut coordinate: T, mut seed: u32) -> f32 {
        coordinate *= self.frequency;
        let mut sum: f32 = 0.0;
        let mut max: f32 = 0.0;
        let mut amplitude = 1.0;
        for _ in 0..self.octaves {
            let mut noise = self.sampler.sample(coordinate, seed);
            max += amplitude;
            match self.fractal_type {
                FractalType::None => {
                    return noise;
                },
                FractalType::Fbm => {
                    sum += amplitude * noise;
                    amplitude *= lerp(1.0, 0.5 * (noise + 1.0).min(2.0), self.strength);
                },
                FractalType::Ridged => {
                    noise = noise.abs();
                    sum += amplitude * (noise * -2.0 + 1.0);
                    amplitude *= lerp(1.0, 1.0 - noise, self.strength);
                },
                FractalType::PingPong(strength) => {
                    let mut t = (noise + 1.0) * strength;
                    t -= (t * 0.5).floor() * 2.0;
                    noise = if t < 1.0 { t }else{ 2.0 - t };
                    sum += amplitude * (noise - 0.5) * 2.0;
                    amplitude *= lerp(1.0, noise, self.strength);
                }
            }
            seed += 1;
            coordinate *= self.lacunarity;
            amplitude *= self.gain;
        }
        sum / max
    }
}