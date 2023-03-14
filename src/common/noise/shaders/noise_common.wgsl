#define_import_path bevy_noise::common

const TAU: f32 = 6.283185307179586;

fn sample_disk(random: vec2<f32>, normal: vec3<f32>) -> vec3<f32> {
    let radius = sqrt(random.x);
    let angle = random.y * TAU;
    let position = vec2<f32>(radius * cos(angle), radius * sin(angle));
    let bitangent = cross(normalize(normal.yzx), normal);
    let tangent = cross(bitangent, normal);
    return tangent * position.x + bitangent * position.y;
}

fn sample_sphere(random: vec2<f32>) -> vec3<f32> {
    let angle = random.x * TAU;
    let z = random.y * 2.0 - 1.0;
    let u = sqrt(1.0 - z * z);
    let x = cos(angle) * u;
    let y = sin(angle) * u;
    return vec3<f32>(x, y, z);
}