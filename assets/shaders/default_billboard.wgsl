#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_types

struct Vertex {
#ifdef VERTEX_POSITIONS
    @location(0) position: vec3<f32>,
#endif
#ifdef VERTEX_NORMALS
    @location(1) normal: vec3<f32>,
#endif
#ifdef VERTEX_UVS
    @location(2) uv: vec2<f32>,
#endif
#ifdef VERTEX_TANGENTS
    @location(3) tangent: vec4<f32>,
#endif
#ifdef VERTEX_COLORS
    @location(4) color: vec4<f32>,
#endif
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    #import bevy_pbr::mesh_vertex_output
#ifdef VERTEX_COLOR_MASK
    @location(5) color_mask: vec4<f32>,
#endif
};

@group(2) @binding(0)
var<uniform> mesh: Mesh;
#import bevy_pbr::mesh_functions

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

    var model = mat4x4<f32>(
        vec4<f32>(mesh.model[0].xyz, 0.0),
        vec4<f32>(mesh.model[1].xyz, 0.0),
        vec4<f32>(mesh.model[2].xyz, 0.0),
        vec4<f32>(mesh.model[3].xyz, 1.0),
    );
    var inverse_transpose = mat3x3<f32>(
        mesh.inverse_transpose_model[0].xyz,
        mesh.inverse_transpose_model[1].xyz,
        mesh.inverse_transpose_model[2].xyz
    );
    var sign_determinant_model = sign_determinant_model_3x3();
#ifdef VERTEX_COLOR_MASK
    out.color_mask = vec4<f32>(mesh.model[0].w, mesh.model[1].w, mesh.model[2].w, mesh.model[3].w);
#endif

#ifdef VERTEX_POSITIONS
#ifdef BILLBOARD
    let translation = model[3].xyz;
    let inverse_model = transpose(inverse_transpose);
#ifdef SCREEN_ALIGNED
    let forward = -view.view[2].xyz;
#else
    let forward = normalize(translation.xyz - view.world_position);
#endif
    let axis_y = normalize(inverse_model * view.view[1].xyz);
    let axis_x = normalize(cross(axis_y, inverse_model * -forward));
    let rotation_matrix = mat3x3<f32>(axis_x, axis_y, cross(axis_x, axis_y));

    out.world_position = model * vec4<f32>(rotation_matrix * vertex.position, 1.0);
#else
    out.world_position = model * vec4<f32>(vertex.position, 1.0);
#endif
    out.clip_position = view.view_proj * out.world_position;
#endif
#ifdef VERTEX_NORMALS
    out.world_normal = normalize(inverse_transpose * vertex.normal);
#endif
#ifdef VERTEX_TANGENTS
    out.world_tangent = vec4<f32>(
        normalize(mat3x3<f32>(model[0].xyz,model[1].xyz,model[2].xyz) * vertex.tangent.xyz),
        vertex.tangent.w * sign_determinant_model
    );
#endif
#ifdef VERTEX_UVS
    out.uv = vertex.uv;
#endif
#ifdef VERTEX_COLORS
    out.color = vertex.color;
#endif
    return out;
}

struct FragmentInput {
    @builtin(front_facing) is_front: bool,
    @builtin(position) frag_coord: vec4<f32>,
    #import bevy_pbr::mesh_vertex_output
#ifdef VERTEX_COLOR_MASK
    @location(5) color_mask: vec4<f32>,
#endif
};


struct MaterialAttributes {
    uv_transform: vec4<f32>,
    diffuse: vec4<f32>,
    alpha_threshold: f32,
};
@group(1) @binding(0)
var<uniform> material: MaterialAttributes;
#ifdef DIFFUSE_MAP
@group(1) @binding(1)
var diffuse_texture: texture_2d<f32>;
@group(1) @binding(2)
var diffuse_sampler: sampler;
#endif

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    var color: vec4<f32> = material.diffuse;
#ifdef VERTEX_UVS
#ifdef DIFFUSE_MAP
    color *= textureSample(diffuse_texture, diffuse_sampler, in.uv);
#endif
#endif

#ifdef ALPHA_MASK
    color *= smoothstep(material.alpha_threshold, 1.0, color.a);
#endif

#ifdef VERTEX_COLOR_MASK
    color *= in.color_mask;
#endif
    return color;
}

