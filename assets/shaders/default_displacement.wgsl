#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_instance_bindings

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    return vertex_output_from(vertex);
}

struct MaterialAttributes {
    displacement: f32,
    chromatic_aberration: f32,
}
@group(1) @binding(0)
var<uniform> material: MaterialAttributes;
#ifdef DISPLACEMENT_MAP
@group(1) @binding(1)
var displacement_texture: texture_2d<f32>;
@group(1) @binding(2)
var displacement_sampler: sampler;
#endif

struct FragmentInput {
    #import bevy_pbr::mesh_vertex_output
#ifdef VERTEX_COLOR_MASK
    @location(5) color_mask: vec4<f32>,
#endif
};

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
#ifdef DISPLACEMENT_MAP
    var sample: vec4<f32> = textureSample(displacement_texture, displacement_sampler, in.uv);
#ifdef VERTEX_TANGENTS
    var uv_offset = vec2<f32>(sample.rg * 2.0 - 1.0);
    let T: vec3<f32> = in.world_tangent.xyz;
    let B: vec3<f32> = in.world_tangent.w * cross(in.world_normal, T);
    let direction = uv_offset.x * T - uv_offset.y * B;
    uv_offset = vec2<f32>(
        dot(direction, view.view[0].xyz),
        -dot(direction, view.view[1].xyz),
    );
    var displacement = vec4<f32>(uv_offset, sample.ba);
#else
    var displacement = vec4<f32>(sample.rg * 2.0 - 1.0, sample.ba);
#endif
    
#else
    let uv_offset = vec2<f32>(
        dot(in.world_normal, view.view[0].xyz),
        -dot(in.world_normal, view.view[1].xyz)
    );
    var displacement = vec4<f32>(uv_offset, 1.0, 1.0);
#endif
#ifdef FRESNEL
    let V = normalize(view.world_position.xyz - in.world_position.xyz);
    let NdotV = max(dot(in.world_normal, V), 0.0);
    let fresnel = clamp(1.0 - NdotV, 0.0, 1.0);
    displacement *= fresnel;
#endif
#ifdef VERTEX_COLOR_MASK
    displacement *= in.color_mask.a;
#endif

    displacement *= vec4<f32>(material.displacement, material.displacement, material.chromatic_aberration, 1.0);
    return vec4<f32>(displacement.rg * 0.5 + 0.5, displacement.ba);
}