#import bevy_pbr::mesh_view_bindings

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

#import bevy_pbr::mesh_types
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
    let inverse_model = transpose(inverse_transpose);

    let side = step(0.5,vertex.uv.x) * 2.0 - 1.0;
    let local_origin = vertex.position + side * vertex.tangent.xyz * vertex.tangent.w;
    let world_origin = model * vec4<f32>(local_origin, 1.0);

#ifdef SCREEN_ALIGNED
    let forward = -view.view[2].xyz;
#else
    let forward = normalize(world_origin.xyz - view.world_position);
#endif
    let T = vertex.tangent.xyz;
    let N = vertex.normal;
    let B = cross(T, N);
#ifdef BILLBOARD_SPHERICAL
    let axis_y = inverse_model * view.view[1].xyz;
#else
    let axis_y = normalize(cross(T, N));
#endif
    let axis_x = normalize(cross(axis_y, inverse_model * forward));
//https://gamedev.stackexchange.com/questions/188636/cylindrical-billboarding-around-an-arbitrary-axis-in-geometry-shader
    let rotation_matrix = mat3x3<f32>(axis_x, axis_y, cross(axis_x, axis_y));

#ifdef INVERSE_TBN
    let TBN = transpose(mat3x3<f32>(T, B, N));
    let position = local_origin + rotation_matrix * TBN * (vertex.position - local_origin);
#else
    let position = local_origin + rotation_matrix * vec3<f32>(-side * vertex.tangent.w, 0.0, 0.0);
#endif

    out.world_position = model * vec4<f32>(position, 1.0);
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


struct EffectMaterial {
    color: vec4<f32>,
    uv_transform: vec4<f32>,
};

@group(1) @binding(0)
var<uniform> material: EffectMaterial;

struct FragmentInput {
    @builtin(front_facing) is_front: bool,
    @builtin(position) frag_coord: vec4<f32>,
    #import bevy_pbr::mesh_vertex_output
#ifdef VERTEX_COLOR_MASK
    @location(5) color_mask: vec4<f32>,
#endif
};

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    return material.color;
}

