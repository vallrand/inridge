mod helpers;
mod hash;
mod sampler;

pub use sampler::*;
pub use hash::MurMurHash;
pub use helpers::WeightTable;

pub mod value;
pub mod perlin;
pub mod simplex;

use bevy::prelude::*;
use bevy::asset::HandleId;

const SHADER_SOURCE: [&str; 5] = [
    include_str!("./shaders/noise_common.wgsl"),
    include_str!("./shaders/noise_hash.wgsl"),
    include_str!("./shaders/value_noise.wgsl"),
    include_str!("./shaders/noise_simplex.wgsl"),
    include_str!("./shaders/noise_cellular.wgsl"),
];

pub struct NoiseShaderPlugin; impl Plugin for NoiseShaderPlugin {
    fn build(&self, app: &mut App) {
        let mut shaders = app.world.resource_mut::<Assets<Shader>>();
        for shader_source in SHADER_SOURCE {
            let shader = Shader::from_wgsl(shader_source);
            let handle_id = HandleId::random::<Shader>();
            shaders.set_untracked(handle_id, shader);
        }
    }
}