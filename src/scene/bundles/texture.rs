use bevy::prelude::*;
use bevy::render::render_resource::{SamplerDescriptor, AddressMode, FilterMode, TextureFormat};
use bevy::render::texture::ImageSampler;

#[derive(Clone)]
pub struct TextureSamplerOptions {
    format: TextureFormat
}
impl TextureSamplerOptions {
    pub const DEFAULT: TextureSamplerOptions = TextureSamplerOptions{
        format: TextureFormat::Rgba8Unorm
    };
    pub const SRGBA: TextureSamplerOptions = TextureSamplerOptions{
        format: TextureFormat::Rgba8UnormSrgb
    };
    pub fn apply(&self, image: &mut Image){
        image.texture_descriptor.format = self.format;
        image.sampler_descriptor = ImageSampler::Descriptor(SamplerDescriptor {
            address_mode_u: AddressMode::Repeat, address_mode_v: AddressMode::Repeat, address_mode_w: AddressMode::Repeat,
            mag_filter: FilterMode::Linear, min_filter: FilterMode::Linear, mipmap_filter: FilterMode::Linear,
            ..Default::default()
        });
    }
}