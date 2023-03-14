#import bevy_pbr::utils
#import bevy_core_pipeline::fullscreen_vertex_shader

@group(0) @binding(0)
var screen_texture: texture_2d<f32>;
@group(0) @binding(1)
var texture_sampler: sampler;
struct PostProcessSettings {
    intensity: f32,
    chromatic_aberration: f32,
}
@group(0) @binding(2)
var<uniform> settings: PostProcessSettings;

@group(0) @binding(3)
var displacement_texture: texture_2d<f32>;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let displacement = textureSample(displacement_texture, texture_sampler, in.uv);
    let uv_offset = settings.intensity * (displacement.rg * 2.0 - 1.0) * displacement.a;
    let uv_split = settings.chromatic_aberration * displacement.b;

    return vec4<f32>(
        textureSample(screen_texture, texture_sampler, in.uv + uv_offset + vec2<f32>(uv_split, -uv_split)).r,
        textureSample(screen_texture, texture_sampler, in.uv + uv_offset + vec2<f32>(-uv_split, 0.0)).g,
        textureSample(screen_texture, texture_sampler, in.uv + uv_offset + vec2<f32>(0.0, uv_split)).b,
        1.0
    );
}
