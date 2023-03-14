#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_types

struct MaterialAttributes {
    color: vec4<f32>,
    glow_color: vec4<f32>,
    domain: vec2<f32>,
    vertical_fade: vec2<f32>,
    threshold: f32,
};

@group(1) @binding(0)
var<uniform> material: MaterialAttributes;

#import bevy_noise::hash
#import bevy_noise::noise

//https://www.shadertoy.com/view/fdGfRV
fn quad_cell_noise(uv: vec2<f32>, domain: vec2<f32>, inv_width: f32, time_scale: f32) -> f32 {
    var cell_uv: vec2<f32> = uv * domain;
    let noise_x = hash11_f32(pcg11(u32(cell_uv.x)));
    cell_uv = vec2<f32>(cell_uv.x, cell_uv.y * mix(1.0, 2.0, noise_x) + mix(1.0, 4.0, noise_x) * (domain.y + time_scale * globals.time));
    let noise_y = hash11_f32(pcg11(u32(cell_uv.y)));
    let noise_cell = hash22_f32(pcg22(vec2<u32>(cell_uv)));
    let f = fract(cell_uv);
    let sdf = vec2<f32>(abs((f.x + 0.5 * (noise_y - 0.5)) * 2.0 - 1.0), 1.0 - f.y);
    return max(0.0, 1.0 - sdf.x * inv_width) * sdf.y * step(0.5, noise_cell.x);
}

//https://www.shadertoy.com/view/MlfXzN
fn char_matrix(uv: vec2<f32>, grid: f32, margin: vec2<f32>, seed: f32) -> vec2<f32> {
    let i = floor(uv) + vec2<f32>(1.0,0.0);
    let f = fract(uv);
    let center = 0.5 - f;
    let fade = 1.0 - dot(center, center) * 4.0;

    let border = step(margin, f) * step(margin, 1.0 - f);
    let char: f32 = step(0.5, hash21(i * seed + floor(f * grid))) * border.x * border.y;

    return vec2<f32>(2.0 * fade * char, hash21(i));
}

struct FragmentInput {
    #import bevy_pbr::mesh_vertex_output
};

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    var uv: vec2<f32> = in.uv;
    var output_color: vec4<f32> = vec4<f32>(0.0);

    let glow = smoothstep(-1.0, 0.0, uv.y) * smoothstep(material.vertical_fade.x, 0.0, uv.y);
    let scan = abs(fract(4.0 * uv.y + globals.time) * 2.0 - 1.0);

    let border = 1.0 - min(1.0, abs(uv.y * 4.0));
    output_color += material.threshold * border * border * material.glow_color;
    output_color += material.threshold * scan * scan * glow * mix(material.glow_color, material.color, 2.0 * -uv.y);

    let char = char_matrix(uv * material.domain, 5.0, vec2<f32>(0.2,0.1), 17.0).x;
    let mask = quad_cell_noise(uv, vec2<f32>(material.domain.x, 1.0), 0.5, 0.2);
    let fade = smoothstep(0.0, material.vertical_fade.x, uv.y) * (1.0 - smoothstep(material.vertical_fade.y, 1.0, uv.y));
    output_color = mix(output_color, material.color, step(mask, material.threshold) * fade * char * mask * 2.0);

    return output_color;
}