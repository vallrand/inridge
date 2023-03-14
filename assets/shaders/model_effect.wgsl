#import bevy_pbr::mesh_view_bindings
#define VERTEX_DEFORMATION
#define VERTEX_COLOR_MASK
#define FRAGMENT_LOCAL_POSITION
#import bevy_pbr::mesh_instance_bindings

struct MaterialAttributes {
    uv_transform: vec4<f32>,
    emission: f32,
    noise_domain: vec3<f32>,
    alpha_threshold: f32,

    color_shift: vec4<f32>,
    scanline_color: vec4<f32>,
    scanline_width: vec4<f32>,

    dissolve_color: vec4<f32>,
    dissolve_plane: vec4<f32>,
    dissolve_offset: vec2<f32>,
};
@group(1) @binding(6)
var<uniform> material: MaterialAttributes;

#import bevy_noise::common
#import bevy_noise::simplex

fn deform_vertex(model_position: vec3<f32>, vertex: Vertex, out: ptr<function, VertexOutput>) -> vec3<f32> {
    var position = model_position;
#ifdef DAMAGE
    let damaged_percent = (*out).color_mask.g;

    let sample = wrapped_simplex_noise3(position * material.noise_domain, vec3<f32>(0.0), 0.0);
    let perturb = normalize(sample.gradient) * sample.noise;
    position += perturb * 0.25 * damaged_percent;
    position *= mix(1.0, 0.8, abs(sample.noise) * damaged_percent);
#endif
    return position;
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out = vertex_output_from(vertex);

#ifdef DISSOLVE
    let threshold = dot(material.dissolve_plane.xyz, out.world_position.xyz) - material.dissolve_plane.w;
    let dissolve = smoothstep(threshold + material.dissolve_offset.x, threshold, material.alpha_threshold);

    out.world_position += vec4<f32>(material.dissolve_plane.xyz * 4.0 * material.dissolve_offset.y * dissolve, 0.0);
    out.clip_position = view.view_proj * out.world_position;
#endif

    return out;
}

#ifdef FRAGMENT

#import shader_template::layered_bindings

fn greyscale(color: vec3<f32>) -> f32 { return dot(color, vec3<f32>(0.2126, 0.7152, 0.0722)); }

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    var pbr_input = calculate_fragment(in, material.uv_transform);
    pbr_input.material.emissive *= material.emission;

    pbr_input.material.base_color = mix(pbr_input.material.base_color, pbr_input.material.base_color.gbra, 1.0 - material.color_shift.x);
    pbr_input.material.emissive = mix(pbr_input.material.emissive, pbr_input.material.emissive.gbra, 1.0 - material.color_shift.x);

    let grey = greyscale(pbr_input.material.base_color.rgb);
    pbr_input.material.base_color = mix(pbr_input.material.base_color, vec4<f32>(vec3<f32>(grey*grey),1.0), in.color_mask.r);
    pbr_input.material.emissive *= 1.0 - in.color_mask.r;

    let line = 1.0 - fract(in.model_position.y * material.scanline_width.z + globals.time * material.scanline_width.w);
    let scanline = smoothstep(0.0, material.scanline_width.x, line) * smoothstep(material.scanline_width.y, material.scanline_width.x, line);
    pbr_input.material.emissive += in.color_mask.b * scanline * material.scanline_color;

#ifdef DAMAGE
    let damaged_percent = in.color_mask.g;
    var damage_threshold = 0.5 + 0.5 * simplex_noise3(in.model_position * material.noise_domain);
    damage_threshold = smoothstep(damage_threshold, damage_threshold + 0.2, damaged_percent);

    pbr_input.material.base_color *= 1.0 - damage_threshold;
    pbr_input.material.emissive += vec4<f32>(2.0,0.2,0.4,1.0) * smoothstep(0.8,1.0, damage_threshold);
    pbr_input.material.emissive *= 1.0 - damage_threshold;
    pbr_input.occlusion += damage_threshold;
#endif

    pbr_input.material.emissive += max(0.0, in.color_mask.a - 1.0) * material.dissolve_color;

    var output_color: vec4<f32> = pbr(pbr_input);
#ifdef DISSOLVE
    let noise_uv = in.model_position * material.noise_domain;
    let noise = 0.5+0.5*simplex_noise3(vec3<f32>(noise_uv.x, 128.0 * floor(noise_uv.y), noise_uv.z) + 8.0 * grey);
    let threshold = noise * material.dissolve_offset.y + dot(material.dissolve_plane.xyz, in.world_position.xyz) - material.dissolve_plane.w;
    if(material.alpha_threshold < threshold){ discard; }
    let dissolve = smoothstep(threshold, threshold + material.dissolve_offset.x, material.alpha_threshold);
    output_color = mix(material.dissolve_color, output_color, dissolve);
#endif

    if(fog.mode != FOG_MODE_OFF){ output_color = apply_fog(output_color, in.world_position.xyz, view.world_position.xyz); }
    return output_color;
}
#endif