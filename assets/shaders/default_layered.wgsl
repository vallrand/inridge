#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_instance_bindings

struct MaterialAttributes {
    uv_transform: vec4<f32>,
    emission: f32,
};
@group(1) @binding(6)
var<uniform> material: MaterialAttributes;

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    return vertex_output_from(vertex);
}

#import shader_template::layered_bindings

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    var pbr_input = calculate_fragment(in, material.uv_transform);
    pbr_input.material.emissive *= material.emission;

    var output_color: vec4<f32> = pbr(pbr_input);

    if(fog.mode != FOG_MODE_OFF){ output_color = apply_fog(output_color, in.world_position.xyz, view.world_position.xyz); }
    return output_color;
}