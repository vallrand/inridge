#import bevy_pbr::mesh_view_bindings
#define VERTEX_COLOR_MASK
#import bevy_pbr::mesh_instance_bindings
#ifdef DEPTH_MASK
#import bevy_pbr::prepass_utils
#endif

struct MaterialAttributes {
    color: vec4<f32>,
    fresnel_mask: f32,
    depth_mask: f32,
}
@group(1) @binding(0)
var<uniform> material: MaterialAttributes;
@group(1) @binding(1)
var diffuse_texture: texture_2d<f32>;
@group(1) @binding(2)
var diffuse_sampler: sampler;

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    return vertex_output_from(vertex);
}

struct FragmentInput {
    @builtin(front_facing) is_front: bool,
    @builtin(position) frag_coord: vec4<f32>,
    @builtin(sample_index) sample_index: u32,
    #import bevy_pbr::mesh_vertex_output
#ifdef VERTEX_COLOR_MASK
    @location(5) color_mask: vec4<f32>,
#endif
};

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    var color: vec4<f32> = material.color;
#ifdef VERTEX_UVS
#ifdef DIFFUSE_MAP
    color *= textureSample(diffuse_texture, diffuse_sampler, in.uv).r;
#endif
#endif

#ifdef FRESNEL_MASK
    let V = normalize(view.world_position.xyz - in.world_position.xyz);
    let NdotV = abs(dot(in.world_normal, V));
    let fresnel = clamp(1.0 - NdotV, 0.0, 1.0);
    color *= mix(1.0, fresnel * fresnel, material.fresnel_mask);
#endif

#ifdef DEPTH_MASK
    let depth = prepass_depth(in.frag_coord, in.sample_index);
    let intersection = smoothstep(0.0002, 0.0, abs(in.frag_coord.z - depth));
    color += material.color * intersection * material.depth_mask;
#endif

#ifdef VERTEX_COLOR_MASK
    color *= in.color_mask;
#endif
    return color;
}