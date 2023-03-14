#import bevy_render::view
#import bevy_render::globals

@group(0) @binding(0)
var<uniform> view: View;
@group(0) @binding(1)
var<uniform> globals: Globals;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
};

@vertex
fn vertex(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let clip_position = vec4(
        f32(vertex_index & 1u), f32((vertex_index >> 1u) & 1u), 0.25, 0.5
    ) * 4.0 - vec4(1.0);
    let world_position_homogeneous = view.inverse_view_proj * vec4(clip_position.xy, 1.0, 1.0);
    let world_position = world_position_homogeneous.xyz / world_position_homogeneous.w;
    return VertexOutput(clip_position, world_position);
}

struct MaterialAttributes {
    color: vec4<f32>,
}
@group(0) @binding(2)
var<uniform> material: MaterialAttributes;

fn field(uvw: vec3<f32>, iterations: i32, offset: f32) -> f32 {
    var p: vec3<f32> = uvw;
    var strength: f32 = 7.0;
    let shift: vec3<f32> = vec3<f32>(-0.5, -0.8 + 0.2 * sin(offset * 0.2 + 2.0), -1.1 + 0.3 * cos(offset * 0.15));
	var accumulated: f32 = 0.;
	var prev: f32 = 0.;
	var total_weight: f32 = 0.;
	for(var i: i32 = 0; i < iterations; i += 1) {
		let mag: f32 = dot(p, p);
		p = abs(p) / mag + shift;
		let w: f32 = exp(-f32(i) / 7.);
		accumulated += w * exp(-strength * pow(abs(mag - prev), 2.3));
		total_weight += w;
		prev = mag;
	}
	return max(0., 5. * accumulated / total_weight - .7);
}

fn kaliset(uvw: vec3<f32>, iterations: i32, param: f32) -> f32 {
    var p: vec3<f32> = uvw;
    var pa: f32 = 0.0; var a: f32 = 0.0;
    for(var i: i32 = 0; i<iterations; i += 1) { 
        p=abs(p)/dot(p,p)-param;
        a+=abs(length(p)-pa);
        pa=length(p);
    }
    return a;
}

fn modv3f(x: vec3<f32>, period: vec3<f32>) -> vec3<f32> { return x - floor(x / period) * period; }

fn sample_skybox(origin: vec3<f32>, ray: vec3<f32>) -> vec4<f32> {
    let period: f32 = 0.8;
    let volsteps: i32 = 4;
    let stepsize: f32 = 0.1;
	
    var color_stars = vec3<f32>(0.0);
    var color_clouds = vec3<f32>(0.0);
    var s: f32 = 0.6;
	
	for(var r: i32 = 0; r < volsteps; r += 1){
        var p: vec3<f32> = origin + s*ray * 4.0;
        p = abs(vec3<f32>(period)-modv3f(p,vec3<f32>(period*2.)));

		let v1 = field(p, 6, 0.5 * globals.time);
        let v0 = kaliset(p, 4, 0.8);
		
		let fade = pow(0.4, f32(r));
		
		color_stars += vec3<f32>(s*s*s*s,s*s,s*s*s)*v0*v0 * fade;
		color_clouds += vec3<f32>(v1,v1*v1*v1,v1*v1*v1) * fade;
		
		s += stepsize;
	}
	return vec4<f32>(color_stars*0.004 + 0.01 * color_clouds, 1.0);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let ray_direction = in.world_position - view.world_position;
    var color: vec4<f32> = sample_skybox(vec3<f32>(0.0), normalize(ray_direction * vec3<f32>(1.0, 1.0, -1.0)));
    return color * material.color;
}