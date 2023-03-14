#define_import_path bevy_noise::hash

fn pcg11(v: u32) -> u32 {
    var v = v * 747796405u + 2891336453u;
	v = ((v >> ((v >> 28u) + 4u)) ^ v) * 277803737u;
	return (v >> 22u) ^ v;
}
fn pcg22(v: vec2<u32>) -> vec2<u32> {
    var v = v * 1664525u + 1013904223u;

    v.x += v.y * 1664525u;
    v.y += v.x * 1664525u;

    v = v ^ (v>>16u);

    v.x += v.y * 1664525u;
    v.y += v.x * 1664525u;

    return v ^ (v>>16u);
}
fn pcg33(v: vec3<u32>) -> vec3<u32> {
    var v = v * 1664525u + 1013904223u;

    v.x += v.y*v.z;
    v.y += v.z*v.x;
    v.z += v.x*v.y;

    v ^= v >> 16u;

    v.x += v.y*v.z;
    v.y += v.z*v.x;
    v.z += v.x*v.y;

    return v;
}
fn pcg44(v: vec4<u32>) -> vec4<u32> {
    var v = v * 1664525u + 1013904223u;
    
    v.x += v.y*v.w;
    v.y += v.z*v.x;
    v.z += v.x*v.y;
    v.w += v.y*v.z;
    
    v ^= v >> 16u;
    
    v.x += v.y*v.w;
    v.y += v.z*v.x;
    v.z += v.x*v.y;
    v.w += v.y*v.z;
    
    return v;
}
fn pcg21(v: vec2<u32>) -> u32 { return pcg11(19u * v.x + 47u * v.y + 101u); }
fn pcg31(v: vec3<u32>) -> u32 { return pcg11(19u * v.x + 47u * v.y + 101u * v.z + 131u); }
fn pcg41(v: vec4<u32>) -> u32 { return pcg11(19u * v.x + 47u * v.y + 101u * v.z + 131u * v.w + 173u); }

fn hash11_f32(hash: u32) -> f32 { return f32(hash) * (1.0/f32(0xffffffffu)); }
fn hash22_f32(hash: vec2<u32>) -> vec2<f32> { return vec2<f32>(hash) * (1.0/f32(0xffffffffu)); }
fn hash33_f32(hash: vec3<u32>) -> vec3<f32> { return vec3<f32>(hash) * (1.0/f32(0xffffffffu)); }

fn hash11(v: f32) -> f32 { return hash11_f32(pcg11(bitcast<u32>(v))); }
fn hash21(v: vec2<f32>) -> f32 { return hash11_f32(pcg21(bitcast<vec2<u32>>(v))); }
fn hash31(v: vec3<f32>) -> f32 { return hash11_f32(pcg31(bitcast<vec3<u32>>(v))); }
fn hash41(v: vec4<f32>) -> f32 { return hash11_f32(pcg41(bitcast<vec4<u32>>(v))); }

fn hash22(v: vec2<f32>) -> vec2<f32> { return hash22_f32(pcg22(bitcast<vec2<u32>>(v))); }
fn hash33(v: vec3<f32>) -> vec3<f32> { return hash33_f32(pcg33(bitcast<vec3<u32>>(v))); }