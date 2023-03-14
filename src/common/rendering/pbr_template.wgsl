#define_import_path bevy_pbr::pbr_template

#import bevy_pbr::pbr_types
#import bevy_pbr::utils
#import bevy_pbr::clustered_forward
#import bevy_pbr::lighting
#import bevy_pbr::shadows
#import bevy_pbr::fog
#import bevy_pbr::pbr_functions
#import bevy_pbr::pbr_ambient

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

fn cotangent_frame(normal: vec3<f32>, position: vec3<f32>, uv: vec2<f32>) -> mat3x3<f32> {
    let p_dx = dpdx(position);
    let p_dy = dpdy(position);
    let uv_dx = dpdx(uv);
    let uv_dy = dpdy(uv);

    let ortho_dy = cross(p_dy, normal);
    let ortho_dx = cross(normal, p_dx);
    let tangent = ortho_dy * uv_dx.x + ortho_dx * uv_dy.x;
    let binormal = ortho_dy * uv_dx.y + ortho_dx * uv_dy.y;
    let inv_max = inverseSqrt(max(dot(tangent, tangent), dot(binormal, binormal)));
    return mat3x3<f32>(tangent * inv_max, binormal * inv_max, normal);
}

fn normal_mapping(
    world_normal: vec3<f32>,
    local_normal: vec3<f32>,
#ifdef VERTEX_TANGENTS
    world_tangent: vec4<f32>,
#else
    world_position: vec3<f32>,
#endif
#ifdef VERTEX_UVS
    uv: vec2<f32>,
#endif
) -> vec3<f32> {
    var N: vec3<f32> = world_normal;
#ifdef VERTEX_TANGENTS
    var T: vec3<f32> = world_tangent.xyz;
    var B: vec3<f32> = world_tangent.w * cross(N, T);
    N = local_normal.x * T + local_normal.y * B + local_normal.z * N;
#else ifdef VERTEX_UVS
    let TBN = cotangent_frame(world_normal, world_position, uv);
    N = TBN * local_normal;
#endif
    return normalize(N);
}

fn pbr_input_from(in: FragmentInput) -> PbrInput {
    var pbr_input: PbrInput = pbr_input_new();

#ifdef VERTEX_COLORS
    pbr_input.material.base_color = pbr_input.material.base_color * in.color;
#endif

    pbr_input.frag_coord = in.frag_coord;
    pbr_input.world_position = in.world_position;
    pbr_input.world_normal = prepare_world_normal(
        in.world_normal,
        (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u,
        in.is_front,
    );

    pbr_input.is_orthographic = view.projection[3].w == 1.0;
    pbr_input.flags = mesh.flags;

    pbr_input.N = apply_normal_mapping(
        pbr_input.material.flags,
        pbr_input.world_normal,
#ifdef VERTEX_TANGENTS
#ifdef STANDARDMATERIAL_NORMAL_MAP
        in.world_tangent,
#endif
#endif
        in.uv,
    );
    pbr_input.V = calculate_view(in.world_position, pbr_input.is_orthographic);

    return pbr_input;
}