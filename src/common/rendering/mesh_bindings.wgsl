#define_import_path bevy_pbr::mesh_instance_bindings

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
#ifdef SKINNED
    @location(5) joint_indices: vec4<u32>,
    @location(6) joint_weights: vec4<f32>,
#endif
#ifdef INSTANCING
    @location(7) instance_row_x: vec4<f32>,
    @location(8) instance_row_y: vec4<f32>,
    @location(9) instance_row_z: vec4<f32>,
    @location(10) instance_row_w: vec4<f32>,
#endif
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    #import bevy_pbr::mesh_vertex_output
#ifdef VERTEX_COLOR_MASK
    @location(5) color_mask: vec4<f32>,
#endif
#ifdef FRAGMENT_LOCAL_POSITION
    @location(6) model_position: vec3<f32>,
#endif
};

#import bevy_pbr::mesh_types

#ifdef INSTANCING
fn calculate_inverse_transpose(model: mat4x4<f32>, sign: ptr<function, f32>) -> mat3x3<f32> {
    let x = cross(model[1].xyz, model[2].xyz);
    let y = cross(model[2].xyz, model[0].xyz);
    let z = cross(model[0].xyz, model[1].xyz);
    let det = dot(model[2].xyz, z);
    *sign = f32(bool(det >= 0.0)) * 2.0 - 1.0;
    return mat3x3<f32>(x / det, y / det, z / det);
}
#else
@group(2) @binding(0)
var<uniform> mesh: Mesh;
#ifdef SKINNED
@group(2) @binding(1)
var<uniform> joint_matrices: SkinnedMesh;
#import bevy_pbr::skinning
#endif
#import bevy_pbr::mesh_functions
#endif

fn vertex_output_from(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    var sign_determinant_model: f32 = 1.0;
#ifdef INSTANCING
    var model = transpose(mat4x4<f32>(
        vertex.instance_row_x,
        vertex.instance_row_y,
        vertex.instance_row_z,
        vec4<f32>(0.0, 0.0, 0.0, 1.0),
    ));
    var inverse_transpose = calculate_inverse_transpose(model, &sign_determinant_model);
#ifdef VERTEX_COLOR_MASK
    out.color_mask = vertex.instance_row_w;
#endif
#else
#ifdef SKINNED
    var model = skin_model(vertex.joint_indices, vertex.joint_weights);
    var inverse_transpose = inverse_transpose_3x3(mat3x3<f32>(model[0].xyz, model[1].xyz, model[2].xyz));
#else
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
    sign_determinant_model = sign_determinant_model_3x3();
#endif
#ifdef VERTEX_COLOR_MASK
    out.color_mask = vec4<f32>(mesh.model[0].w, mesh.model[1].w, mesh.model[2].w, mesh.model[3].w);
#endif
#endif

#ifdef VERTEX_NORMALS
    out.world_normal = normalize(inverse_transpose * vertex.normal);
#endif

#ifdef VERTEX_UVS
    out.uv = vertex.uv;
#endif

#ifdef VERTEX_TANGENTS
    out.world_tangent = vec4<f32>(
        normalize(mat3x3<f32>(model[0].xyz,model[1].xyz,model[2].xyz) * vertex.tangent.xyz),
        vertex.tangent.w * sign_determinant_model
    );
#endif

#ifdef VERTEX_COLORS
    out.color = vertex.color;
#endif

#ifdef VERTEX_POSITIONS
    var model_position = vertex.position;
#ifdef VERTEX_DEFORMATION
    model_position = deform_vertex(model_position, vertex, &out);
#endif
#ifdef FRAGMENT_LOCAL_POSITION
    out.model_position = model_position;
#endif
    out.world_position = model * vec4<f32>(model_position, 1.0);
    out.clip_position = view.view_proj * out.world_position;
#endif
    return out;
}