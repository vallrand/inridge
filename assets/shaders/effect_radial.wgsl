#import bevy_sprite::mesh2d_view_bindings
#import bevy_sprite::mesh2d_bindings

struct MaterialAttributes {
    inner_color: vec4<f32>,
    outer_color: vec4<f32>,
    radius: vec2<f32>,
    sectors: u32,
    fraction: f32,
    padding: vec2<f32>,
    grid_resolution: vec2<f32>,
};

@group(1) @binding(0)
var<uniform> material: MaterialAttributes;

struct FragmentInput {
    @builtin(front_facing) is_front: bool,
    @builtin(position) position: vec4<f32>,
    #import bevy_sprite::mesh2d_vertex_output
};
//https://www.shadertoy.com/view/Nlc3zf
fn box_sdf(center: vec2<f32>, halfsize: vec2<f32>, radius: f32) -> f32 {
    let position = abs(center) - halfsize + radius;
    return length(max(position, vec2<f32>(0.0))) + min(max(position.x, position.y), 0.0) - radius;
}

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    var output_color: vec4<f32> = vec4<f32>(0.0);

#ifdef VERTEX_COLORS
    let sectors = in.color.x;
    let fraction = in.color.y;
#endif

    let TAU: f32 = 6.283185307179586;
    let uv = in.uv * 2.0 - 1.0;
    var polar = vec2<f32>(atan2(-uv.x,uv.y) / TAU + 0.5, length(uv));
    let cutoff = smoothstep(polar.x, polar.x + material.padding.y, fraction * (1.0 + material.padding.y));
    let grid = abs(fract(polar * material.grid_resolution) * 2.0 - 1.0);
    polar = vec2<f32>(fract(polar.x * sectors), polar.y);

    let radius = vec2<f32>(0.5*(material.radius.x + material.radius.y), abs(material.radius.x - material.radius.y));
    let unskew = vec2<f32>(TAU / sectors, 1.0);
    let border = box_sdf(unskew * (polar - vec2<f32>(0.5, radius.x)), unskew * vec2<f32>(0.5 - material.padding.x * sectors, radius.y), 0.1);
    let glow = 0.01 / abs(border);
    var fill = step(border, 0.0);
    fill -= 0.5 - fill * smoothstep(-0.2,0.0,border) * smoothstep(0.6,1.0,max(grid.x,grid.y));

    output_color = max(0.0, fill) * mix(material.inner_color, material.outer_color, cutoff);
    output_color += material.outer_color * glow;

    return output_color;
}