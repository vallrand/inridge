#import bevy_sprite::mesh2d_view_bindings
#import bevy_sprite::mesh2d_bindings

struct MaterialAttributes {
    color: vec4<f32>,
    alpha_cutoff: f32,
    fraction: f32,
    fraction_width: f32,
};

@group(1) @binding(0)
var<uniform> material: MaterialAttributes;
@group(1) @binding(1)
var texture: texture_2d<f32>;
@group(1) @binding(2)
var texture_sampler: sampler;

#import bevy_noise::hash
#import bevy_noise::noise

fn from_rotation(theta: f32) -> mat2x2<f32> {
    let c = cos(theta); let s = sin(theta);
    return mat2x2<f32>(c,-s,s,c);
}
fn fbm(uv: vec2<f32>, octaves: i32, lacunarity: f32, gain: f32) -> f32 {
    var m2: mat2x2<f32> = from_rotation(0.6435011087932844);
    var uv = uv;
    var total: f32 = 0.0; var amplitude: f32 = gain;
    for(var i: i32 = 0; i < octaves; i = i + 1){
        total = total + amplitude * abs(noise21(uv)*2. - 1.);
        uv = m2 * uv * lacunarity;
        amplitude = amplitude * gain; 
    }
    return total;
}

struct FragmentInput {
    @builtin(front_facing) is_front: bool,
    @builtin(position) position: vec4<f32>,
    #import bevy_sprite::mesh2d_vertex_output
};

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    var output_color: vec4<f32> = textureSample(texture, texture_sampler, in.uv);
#ifdef VERTEX_COLORS
    output_color = output_color * in.color;
#endif

    var uv = (in.position.xy - view.viewport.xy) / view.viewport.zw;
    let dual = fbm(uv * 16.0, 4, 2.0, 0.5);
    uv = uv + 0.1 * (1.0 - 2.0 * noise21(uv.yx * 18.0 - vec2<f32>(0.2 * globals.time, dual)));
    uv = 12.0 * uv + 0.5 * (1.0 - 2.0*dual) + vec2<f32>(0.4 * globals.time, 0.0);
    var height = fbm(uv, 2, 1.6, 0.6);

    var color = mix(vec3<f32>(0.1,-0.1,0.0), material.color.rgb, 1.5 * height);
    let fraction = smoothstep(material.fraction - material.fraction_width, material.fraction + material.fraction_width, in.uv.x + 0.05 * (1.0 - 2.0 * height));
    color = mix(color, vec3<f32>(max(0.0, height - dual),0.0,0.0), 1.0 - fraction);
    output_color = vec4<f32>(color, smoothstep(0.0, material.alpha_cutoff, output_color.a));

    return output_color;
}