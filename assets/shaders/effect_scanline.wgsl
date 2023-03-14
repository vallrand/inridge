#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_instance_bindings

struct MaterialAttributes {
    color: vec4<f32>,
    uv_transform: vec4<f32>,
    vertical_fade: vec2<f32>,
    line_width: vec4<f32>,
};
@group(1) @binding(0)
var<uniform> material: MaterialAttributes;

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    return vertex_output_from(vertex);
}

struct FragmentInput {
    #import bevy_pbr::mesh_vertex_output
#ifdef VERTEX_COLOR_MASK
    @location(5) color_mask: vec4<f32>,
#endif
};

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    var color: vec4<f32> = material.color;

    let fade = smoothstep(0.0, material.vertical_fade.x, in.uv.y) * (1.0 - smoothstep(material.vertical_fade.y, 1.0, in.uv.y));

    var scanline = 1.0 - fract(in.uv.y * material.line_width.z + globals.time * material.line_width.w);
    scanline = smoothstep(0.0, material.line_width.x, scanline) * smoothstep(material.line_width.y, material.line_width.x, scanline);

    let uv = in.uv * material.uv_transform.zw + material.uv_transform.xy;
    let grid_uv = abs(fract(uv) * 2.0 - 1.0);
#ifdef GRID
    let pattern = smoothstep(mix(1.0,0.5,scanline),1.0,max(grid_uv.x, grid_uv.y));
#else
    let pattern = smoothstep(mix(1.0,0.5,scanline),1.0,grid_uv.y);
#endif

    color *= scanline * pattern * fade;
#ifdef VERTEX_COLOR_MASK
    color *= in.color_mask.a;
#endif
    return color;
}