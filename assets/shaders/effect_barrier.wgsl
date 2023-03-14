#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings
#import bevy_pbr::prepass_utils

struct EffectMaterial {
    color: vec4<f32>,
    fresnel_mask: f32,
}

@group(1) @binding(0)
var<uniform> material: EffectMaterial;

struct FragmentInput {
    @builtin(front_facing) is_front: bool,
    @builtin(position) frag_coord: vec4<f32>,
    @builtin(sample_index) sample_index: u32,
    #import bevy_pbr::mesh_vertex_output
#ifdef VERTEX_COLOR_MASK
    @location(5) color_mask: vec4<f32>,
#endif
};

#import bevy_noise::hash
#import bevy_noise::cellular

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    var color: vec4<f32> = material.color;

    let up = normalize(mesh.model[2].xyz);

    let n0 = wrapped_worley31(in.world_position.xyz * vec3<f32>(4.0) - up * globals.time, vec3<f32>(1024.0));
    color *= smoothstep(0.0, 2.0, n0);

#ifdef FRESNEL_MASK
    let V = normalize(view.world_position.xyz - in.world_position.xyz);
    let NdotV = abs(dot(in.world_normal, V));
    let fresnel = clamp(1.0 - NdotV, 0.0, 1.0);
    color *= mix(1.0, fresnel * fresnel, material.fresnel_mask);
#endif

    let depth = prepass_depth(in.frag_coord, in.sample_index);
    let intersection = smoothstep(0.0002, 0.0, abs(in.frag_coord.z - depth));
    color += material.color * n0 * intersection;

#ifdef VERTEX_COLOR_MASK
    color *= in.color_mask;
#endif
    return color;
}