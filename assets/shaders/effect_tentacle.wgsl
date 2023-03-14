#import bevy_pbr::mesh_view_bindings
#define VERTEX_COLOR_MASK
#define VERTEX_DEFORMATION
#import bevy_pbr::mesh_instance_bindings
#import bevy_pbr::pbr_template

struct MaterialAttributes {
    color: vec4<f32>,
    uv_transform: vec4<f32>,
    reflectance: f32,
    metallic: f32,
    roughness: f32,
};
@group(1) @binding(0)
var<uniform> material: MaterialAttributes;

#import bevy_noise::hash
#import bevy_noise::noise

fn deform_vertex(model_position: vec3<f32>, vertex: Vertex, out: ptr<function, VertexOutput>) -> vec3<f32> {
    var position = model_position;
    let fade = (*out).color_mask.r;
    let v = (*out).uv.y;
    let l = dot(position.xz, position.xz);
    let amplitude: f32 = 0.5 * min(1.0, l) * smoothstep(1.0, 0.0, v);

    position += fade * vec3<f32>(0.0,1.0,0.0) * amplitude * cos(v * 16.0 - 2.0 * globals.time);

    let width: f32 = smoothstep(0.0, 0.2, v) * 0.08;
    position += fade * width * vertex.normal * (0.5 + 0.5 * sin(v * 24.0 - 4.0 * globals.time));
    position += vertex.normal * 0.04 * (1.0 - abs(v * 2.0 - 1.0));

    return position;
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    return vertex_output_from(vertex);
}

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    var pbr_input: PbrInput = pbr_input_from(in);
    let uv = in.uv * material.uv_transform.zw + material.uv_transform.xy;
    let fade = in.color_mask.r;

    let n0 = noise31(in.world_position.xyz * vec3<f32>(8.0)) * 2.0 - 1.0;
    let n1 = wrapped_noise21(in.uv * vec2<f32>(4.0) + n0 + vec2<f32>(0.0, globals.time), vec2<f32>(4.0, 64.0)) * 2.0 - 1.0;
    let color = (1.0 - abs(n1)) * material.color;

    pbr_input.material.reflectance = material.reflectance;
    pbr_input.material.metallic = material.metallic;
    pbr_input.material.perceptual_roughness = material.roughness;
    pbr_input.material.base_color = mix(4.0 * color, color, fade);
    pbr_input.N = normal_mapping(
        pbr_input.world_normal,
        normalize(vec3<f32>(n0,n1,1.0)),
        pbr_input.world_position.xyz,
        in.uv
    );

    return pbr(pbr_input);
}