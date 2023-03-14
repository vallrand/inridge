#define_import_path bevy_noise::cellular

fn modv2f(x: vec2<f32>, period: vec2<f32>) -> vec2<f32> { return x - floor(x / period) * period; }
fn modv3f(x: vec3<f32>, period: vec3<f32>) -> vec3<f32> { return x - floor(x / period) * period; }

fn wrapped_worley31(uv: vec3<f32>, period: vec3<f32>) -> f32 {
    let id = floor(uv); let f = fract(uv);
    var min_dist: f32 = 10000.;
    for(var x: f32 = -1.0; x <= 1.0; x += 1.0){
        for(var y: f32 = -1.0; y <= 1.0; y += 1.0){
            for(var z: f32 = -1.0; z <= 1.0; z += 1.0){
                let offset = vec3<f32>(x, y, z);
                var n = hash33(modv3f(id + offset, period)) * 0.5 + 0.5;
                let d = f - (n + offset);
                min_dist = min(min_dist, dot(d, d));
            }
        }
    }
    return min_dist;
}

fn wrapped_voronoi21(uv: vec2<f32>, period: vec2<f32>) -> vec2<f32> {
    let n = floor(uv); let f = fract(uv);
    var cell: vec3<f32> = vec3<f32>(8.0);
    for(var j: i32 = -1; j <= 1; j = j + 1){
        for(var i: i32 = -1; i <= 1; i = i + 1){
            let g = vec2<f32>(f32(i), f32(j));
            let o = hash22(modv2f(n + g, period));
            let r = g - f + o;
            let d = dot(r, r);
            if(d < cell.x){
                cell = vec3<f32>(d, o);
            }
        }
    }
    return vec2<f32>(sqrt(cell.x), cell.y + cell.z);
}