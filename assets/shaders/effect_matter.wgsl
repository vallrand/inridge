#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings
#import bevy_pbr::pbr_types

#import bevy_pbr::utils
#import bevy_pbr::clustered_forward
#import bevy_pbr::lighting
#import bevy_pbr::pbr_ambient
#import bevy_pbr::shadows
#import bevy_pbr::fog
#import bevy_pbr::pbr_functions

#import bevy_noise::simplex

struct MaterialAttributes {
    diffuse: vec4<f32>,
    emissive: vec4<f32>,
    noise_domain: vec3<f32>,
    flags: u32,
};

@group(1) @binding(0)
var<uniform> material: MaterialAttributes;

struct FragmentInput {
    @builtin(front_facing) is_front: bool,
    @builtin(position) frag_coord: vec4<f32>,
    #import bevy_pbr::mesh_vertex_output
};

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    var pbr_input: PbrInput;

    pbr_input.material.base_color = material.diffuse;
#ifdef VERTEX_COLORS
    pbr_input.material.base_color = pbr_input.material.base_color * in.color;
#endif
    pbr_input.material.reflectance = 0.5;
    pbr_input.material.flags = material.flags;
    pbr_input.material.alpha_cutoff = 0.0;
    pbr_input.material.emissive = material.emissive;
    pbr_input.material.metallic = 0.0;
    pbr_input.material.perceptual_roughness = 0.5;

    pbr_input.frag_coord = in.frag_coord;
    pbr_input.world_position = in.world_position;
    pbr_input.world_normal = prepare_world_normal(in.world_normal, (material.flags & STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u, in.is_front);

    pbr_input.is_orthographic = view.projection[3].w == 1.0;
    pbr_input.N = apply_normal_mapping(material.flags, pbr_input.world_normal,
#ifdef VERTEX_UVS
        in.uv,
#endif
    );
    pbr_input.V = calculate_view(in.world_position, pbr_input.is_orthographic);
    pbr_input.occlusion = 1.0;
    pbr_input.flags = mesh.flags;

    var uvw = in.world_position.xyz * material.noise_domain;
    var total: f32 = 0.0;
    var amplitude: f32 = 1.0;
    for(var i: i32 = 0; i < 3; i += 1){
        uvw += 0.1 * globals.time;
        amplitude *= 0.5;
        let noise = simplex_noise3(uvw);
        total += amplitude * abs(noise);
        uvw = uvw * 1.6 + noise;
    }

    pbr_input.material.emissive = mix(vec4<f32>(0.0), pbr_input.material.emissive, total);

    var output_color: vec4<f32> = pbr(pbr_input);

    if (fog.mode != FOG_MODE_OFF && (material.flags & STANDARD_MATERIAL_FLAGS_FOG_ENABLED_BIT) != 0u) {
        output_color = apply_fog(output_color, in.world_position.xyz, view.world_position.xyz);
    }

    return output_color;
}