#import bevy_pbr::mesh_view_bindings

struct EffectMaterial {
    uv_transform: vec4<f32>,
    color: vec4<f32>,
}

@group(1) @binding(0)
var<uniform> material: EffectMaterial;

struct FragmentInput {
    @builtin(position) frag_coord: vec4<f32>,
    #import bevy_pbr::mesh_vertex_output
#ifdef VERTEX_COLOR_MASK
    @location(5) color_mask: vec4<f32>,
#endif
};

#import bevy_noise::hash
#import bevy_noise::simplex

fn lightning(uv: vec2<f32>, phase: f32, amplitude: f32, width: f32) -> f32 {
    let n0: f32 = (
        simplex_noise2(vec2<f32>(phase + 0., uv.x*0.5)) * 0.18 + 
        simplex_noise2(vec2<f32>(phase + 73., uv.x*2.0)) * 0.09 +
        simplex_noise2(vec2<f32>(phase + 119., uv.x*8.0)) * 0.03
    );
        
    let x = amplitude * (1.0 - abs(uv.x));
    let w = width * mix(0.0001, 0.01, abs(n0));
    return clamp(w / smoothstep(0.0, 1.0, abs(uv.y - n0 * x)), 0.0, 1.0);
}

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    var uv = in.uv * material.uv_transform.zw + material.uv_transform.xy;
    uv = uv.yx * 2.0 - 1.0;
    var color: vec4<f32> = vec4<f32>(0.0);
#ifdef VERTEX_COLORS
    let time = 1.0 * globals.time + 4.0 * in.color.y;
    let mx = uv.x - 2.0 + 4.0 * fract(time);
    let w = lightning(uv, in.color.x + floor(time), 8.0, 1.0 - abs(mx));
    color += 4.0 * material.color * smoothstep(0.5, 1.0, w);
    color += 1.0 * material.color * smoothstep(0.0, 0.5, w);
    color += 0.5 * material.color * smoothstep(0.0, 0.01, w);
    let mask = step(0.5, hash11(floor(time)));
    color *= mask;
#endif
    let fade = smoothstep(1.0, 0.5, abs(in.uv.x));
    color *= fade;

    color = vec4<f32>(color.rgb * color.a, color.a);

#ifdef VERTEX_COLOR_MASK
    color *= in.color_mask;
#endif
    return color;
}

