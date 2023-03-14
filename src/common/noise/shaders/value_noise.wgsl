#define_import_path bevy_noise::noise

fn modv2f(x: vec2<f32>, period: vec2<f32>) -> vec2<f32> { return x - floor(x / period) * period; }
fn modv3f(x: vec3<f32>, period: vec3<f32>) -> vec3<f32> { return x - floor(x / period) * period; }

fn noise11(v: f32) -> f32 {
    let i = floor(v);
    var f = fract(v); f = f * f * (3.0 - 2.0 * f);
    return mix(hash11(i), hash11(i + 1.0), f);
}

fn noise21(v: vec2<f32>) -> f32 {
    let i = floor(v);
    var f = fract(v); f = f * f * (3.0 - 2.0 * f);
    let d = vec2<f32>(0.0, 1.0);
    return mix(mix(
        hash21(i),
        hash21(i + d.yx), f.x), mix(
        hash21(i + d.xy),
        hash21(i + d.yy), f.x), f.y);
}

fn noise31(v: vec3<f32>) -> f32 {
    let i = floor(v);
    var f = fract(v); f = f * f * (3.0 - 2.0 * f);
    let d = vec2<f32>(0.0, 1.0);
    return mix(mix(mix(
        hash31(i+d.xxx), 
        hash31(i+d.yxx),f.x),mix(
        hash31(i+d.xyx), 
        hash31(i+d.yyx),f.x),f.y),mix(mix(
        hash31(i+d.xxy), 
        hash31(i+d.yxy),f.x),mix(
        hash31(i+d.xyy), 
        hash31(i+d.yyy),f.x),f.y),f.z);
}

fn wrapped_noise21(v: vec2<f32>, period: vec2<f32>) -> f32 {
    let i = floor(v);
    var f = fract(v); f = f * f * (3.0 - 2.0 * f);
    let ii = vec4<f32>(modv2f(i, period), modv2f(i + 1.0, period));
    return mix(mix(
        hash21(ii.xy),
        hash21(ii.zy), f.x), mix(
        hash21(ii.xw),
        hash21(ii.zw), f.x), f.y);
}

fn wrapped_noise31(x: vec3<f32>, period: vec3<f32>) -> f32 {
    var i = floor(x);
    var f = fract(x); f = f * f * (3.0 - 2.0 * f);
    let i0 = modv3f(i, period); let i1 = modv3f(i + 1.0, period);
    return mix(mix(mix(
        hash31(vec3<f32>(i0.x,i0.y,i0.z)),
        hash31(vec3<f32>(i1.x,i0.y,i0.z)),f.x),mix(
        hash31(vec3<f32>(i0.x,i1.y,i0.z)), 
        hash31(vec3<f32>(i1.x,i1.y,i0.z)),f.x),f.y),mix(mix(
        hash31(vec3<f32>(i0.x,i0.y,i1.z)), 
        hash31(vec3<f32>(i1.x,i0.y,i1.z)),f.x),mix(
        hash31(vec3<f32>(i0.x,i1.y,i1.z)), 
        hash31(vec3<f32>(i1.x,i1.y,i1.z)),f.x),f.y),f.z);
}