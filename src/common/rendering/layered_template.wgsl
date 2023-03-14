#define_import_path shader_template::layered_bindings

#import bevy_pbr::pbr_types
#import bevy_pbr::utils
#import bevy_pbr::clustered_forward
#import bevy_pbr::lighting
#import bevy_pbr::pbr_ambient
#import bevy_pbr::shadows
#import bevy_pbr::fog
#import bevy_pbr::pbr_functions

@group(1) @binding(0)
var albedo_texture: texture_2d_array<f32>;
@group(1) @binding(1)
var albedo_sampler: sampler;
@group(1) @binding(2)
var normal_texture: texture_2d_array<f32>;
@group(1) @binding(3)
var normal_sampler: sampler;
@group(1) @binding(4)
var roughness_metallic_occlusion_texture: texture_2d_array<f32>;
@group(1) @binding(5)
var roughness_metallic_occlusion_sampler: sampler;

struct FragmentInput {
    @builtin(front_facing) is_front: bool,
    @builtin(position) frag_coord: vec4<f32>,
    #import bevy_pbr::mesh_vertex_output
#ifdef VERTEX_COLOR_MASK
    @location(5) color_mask: vec4<f32>,
#endif
#ifdef FRAGMENT_LOCAL_POSITION
    @location(6) model_position: vec3<f32>,
#endif
};

fn calculate_fragment(in: FragmentInput, uv_transform: vec4<f32>) -> PbrInput {
    let num_layers = textureNumLayers(albedo_texture);
    let layer: i32 = i32(in.uv.y * f32(num_layers)) % num_layers;
    let uv: vec2<f32> = (uv_transform.xy + uv_transform.zw * in.uv) * vec2<f32>(1.0, f32(num_layers));

    var albedo = textureSample(albedo_texture, albedo_sampler, uv, layer);
    var normal = textureSample(normal_texture, normal_sampler, uv, layer);
    var rma = textureSample(roughness_metallic_occlusion_texture, roughness_metallic_occlusion_sampler, uv, layer);

    var pbr_input: PbrInput = pbr_input_new();

    pbr_input.material.base_color = albedo;
    pbr_input.material.metallic = rma.g;
    pbr_input.material.perceptual_roughness = rma.r;
    pbr_input.material.reflectance = 0.5;
    pbr_input.material.emissive = vec4<f32>(rma.a * albedo.rgb, 1.0);
    pbr_input.occlusion = rma.b;
#ifdef INSTANCING
    pbr_input.flags = 1u;
#else
    pbr_input.flags = mesh.flags;
#endif
    pbr_input.frag_coord = in.frag_coord;
    pbr_input.world_position = in.world_position;
    pbr_input.world_normal = prepare_world_normal(
        in.world_normal,
        (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u,
        in.is_front,
    );

    pbr_input.is_orthographic = view.projection[3].w == 1.0;

    var N: vec3<f32> = pbr_input.world_normal;
#ifdef VERTEX_TANGENTS
    let T: vec3<f32> = in.world_tangent.xyz;
    let B: vec3<f32> = in.world_tangent.w * cross(N, T);
    var Nt = normal.rgb * 2.0 - 1.0;
    N = Nt.x * T - Nt.y * B + Nt.z * N;
#endif
    pbr_input.N = normalize(N);
    pbr_input.V = calculate_view(in.world_position, pbr_input.is_orthographic);

    return pbr_input;
}