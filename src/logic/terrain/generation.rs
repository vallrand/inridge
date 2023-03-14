use bevy::prelude::*;
use crate::common::noise::{HeightSampler, FractalSampler, FractalType, simplex::SimplexNoise};
use crate::common::geometry::{unwrap_equirectangular, Icosahedron, sphere::Sphere};
use crate::common::raycast::HitArea;
use super::hexsphere::HexSphere;
use super::grid::MapGrid;

#[derive(Component, serde::Deserialize, Clone, Default)]
pub struct AreaBlueprint {
    pub center: Vec3,
    resolution: usize,
    variants: Vec<f32>,
    noise_frequency: f32,
    noise_octaves: usize,
    seed: u32,
}

impl AreaBlueprint {
    pub fn radius(&self) -> f32 { 1.0 / Icosahedron::circumscribed_tile_radius(self.resolution) }
    pub fn load(&self) -> (Mesh, Transform, HitArea, MapGrid) {
        let mut hexsphere = HexSphere::new(self.resolution, false);
        self.regenerate(&mut hexsphere, self.seed);
        let radius = self.radius();
        (
            Mesh::from(&hexsphere),
            Transform::from_translation(self.center)
            .with_scale(Vec3::splat(radius)),
            HitArea::Sphere(Sphere::default()),
            MapGrid{ graph: hexsphere.graph, tiles: hexsphere.tiles, ..Default::default() },
        )
    }
    fn regenerate(&self, target: &mut HexSphere, seed: u32){
        target.variants = self.variants.len();
        let mut noise = FractalSampler::from(SimplexNoise::<Vec2>::default());
        noise.fractal_type = FractalType::Fbm;
        noise.frequency = (target.graph.len() as f32).sqrt() * self.noise_frequency;
        noise.octaves = self.noise_octaves;
        for (index, tile) in target.tiles.iter_mut().enumerate() {
            let position = tile.transform.translation.into();
            let uv = Vec2::from(unwrap_equirectangular(&position));

            let height = 0.5 + 0.5 * noise.sample(uv, seed);
            tile.variant = self.variants.iter().position(|&threshold|height <= threshold).unwrap_or_default();

            if target.graph.neighbors(index).filter(|list|list.len() == 5).is_some() {
                tile.flags |= MapGrid::BLOCKER;
            }
        }
    }
}