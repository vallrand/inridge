#import bevy_pbr::mesh_view_bindings

struct EffectMaterial {
    uv_transform: vec4<f32>,
    color: vec4<f32>,
}
@group(1) @binding(0)
var<uniform> material: EffectMaterial;

struct FragmentInput {
    @builtin(front_facing) is_front: bool,
    @builtin(position) frag_coord: vec4<f32>,
    #import bevy_pbr::mesh_vertex_output
#ifdef VERTEX_COLOR_MASK
    @location(5) color_mask: vec4<f32>,
#endif
};

#import bevy_noise::hash
#import bevy_noise::noise
const TAU: f32 = 6.283185307179586;

fn fbm(uv: vec3<f32>, octaves: i32, lacunarity: f32, period: vec3<f32>) -> f32 {
    var f: f32 = 0.0; var amplitude: f32 = 0.5;
    var domain = period;
    for(var i: i32 = 0; i < octaves; i += 1){
        f += amplitude * wrapped_noise31(domain * uv, domain);
        domain *= lacunarity;
        amplitude /= lacunarity;
    }
    return f;
}

fn nstep(value: f32, threshold: f32, width: f32) -> f32 {
    let v = value * (1.0 + width);
    return smoothstep(v - width, v, threshold);
}

fn effect_texture(_uv: vec2<f32>, time: f32) -> vec4<f32> {
    let uv = 2.0 * (_uv * 2.0 - 1.0);
    var polar = vec2<f32>(atan2(-uv.x,uv.y) / TAU + 0.5, length(uv));

    polar += vec2<f32>(0.0, -0.6 * time);

    let n0 = fbm(vec3<f32>(polar, 0.2 * globals.time), 4, 2.0, vec3<f32>(16.0, 4.0, 4.0));
    let n1 = fbm(vec3<f32>(uv, polar.y + n0), 3, 2.0, vec3<f32>(4.0, 4.0, 8.0));

    let s0 = smoothstep(1.0, -0.4, polar.y - n0 + 0.1 * n1);
    let s1 = smoothstep(1.5, 0.2, polar.y * n1 + n0);
    
    let cutin = nstep(smoothstep(0.6, 0.0, time), s0, 0.2);
    let cutout = 1.0 - nstep(smoothstep(1.0, 0.5, time), s1, 0.5);
    let fade = nstep(smoothstep(0.6, 0.0, time), s0 * min(n0, 1.0 - n1), 0.6);
    
    let v0 = s0 * cutout * cutin;
    var color = vec4<f32>(0.0);
    color += 4.0 * vec4<f32>(1.0,1.0,1.0,1.0) * smoothstep(0.8,1.0 + time,v0);
    color += 4.0 * material.color * smoothstep(0.0,1.0,v0);
    color = mix(color, vec4<f32>(vec3<f32>(0.2) * n0 * n1,1.0) * v0, fade);
    return color;
}

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    let uv = in.uv * material.uv_transform.zw + material.uv_transform.xy;
    return effect_texture(uv, in.color_mask.a);
}