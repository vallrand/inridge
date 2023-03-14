#import bevy_pbr::mesh_view_bindings

struct LineMaterial {
    color: vec4<f32>,
    head_color: vec4<f32>,
    uv_transform: vec4<f32>,
    vertical_fade: vec2<f32>,
    time_scale: f32,
    iterations: i32,
};

@group(1) @binding(0)
var<uniform> material: LineMaterial;

struct FragmentInput {
    @builtin(front_facing) is_front: bool,
    @builtin(position) frag_coord: vec4<f32>,
    #import bevy_pbr::mesh_vertex_output
#ifdef VERTEX_COLOR_MASK
    @location(5) color_mask: vec4<f32>,
#endif
};

const PI: f32 = 3.141592653589793;
const TAU: f32 = 6.283185307179586;
fn line_wave(uv: vec2<f32>, frequency: f32, phase: f32, max_width: f32) -> f32 {
    var v = 0.25 * cos(uv.x * frequency + 4.0 * globals.time * TAU + phase);
    v *= smoothstep(0.0, 0.5, 1.0 - uv.x);
    v = abs((uv.y + v) * 2.0 - 1.0);
    return smoothstep(max_width * uv.x, 0.0, v) * smoothstep(0.0, 0.2, uv.x);
}

fn effect_texture(uv: vec2<f32>, iterations: i32, shift: f32) -> vec4<f32> {
    let head = vec2<f32>(1.0 - uv.x, uv.y - 0.5);
    let theta = atan2(head.y, head.x);
    let shimmer = 1.0 + 0.5 * sin(abs(theta) * 8.0 + uv.x * 8.0 + (globals.time + shift) * 8.0);
    let distance = 1.0 + 0.1 * cos(theta * 15.0) * shimmer;
    let fade = smoothstep(0.0,0.5,cos(theta));
    let muzzle = fade * max(0.0, 1.0 - ((abs(head.x) + abs(head.y * 2.0)) / distance));

    var line: f32 = 0.0;
    for(var i: i32 = 0; i < iterations; i++){
        let f = f32(i) / f32(iterations) + shift;
        line = max(line, line_wave(uv, PI + 1.5 * PI * f, TAU * f, 0.1));
    }

    var color = vec4<f32>(0.0);
    color += fade * smoothstep(0.0,1.0,line) * vec4<f32>(0.4,0.5,0.2,0.4);
    color += fade * smoothstep(0.8,1.0,line) * vec4<f32>(1.0);
    color += fade * muzzle * material.head_color;
    return color;
}

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    let fade = smoothstep(0.0, material.vertical_fade.x, in.uv.y) * (1.0 - smoothstep(material.vertical_fade.y, 1.0, in.uv.y));
    var uv = in.uv * material.uv_transform.zw + material.uv_transform.xy;
    var shift: f32 = 0.0;
#ifdef VERTEX_COLORS
    shift = in.color.g;
#endif
    return effect_texture(vec2<f32>(uv.y, 0.5 + (uv.x - 0.5) / fade), material.iterations, shift);
}