use bevy::math::Vec3A;
use std::f32::consts::{PI,SQRT_2};

pub fn delta_mod(modulo: f32, lhs: f32, rhs: f32) -> f32 {
    let delta = (rhs - lhs) % modulo;
    2.0 * delta % modulo - delta
}

pub fn unwrap_equirectangular(normal: &Vec3A) -> [f32; 2] {
    let u = 0.5 + 0.5 * normal.z.atan2(normal.x) / PI;
    let v = 0.5 + normal.y.asin() / PI;
    [u*2.0,v]
}

pub fn unwrap_sphere(normal: &Vec3A, diagonal: bool, stereo_quincuncial: bool) -> [f32; 2] {
    let (u0, v0) = if !diagonal {
        (normal.x, normal.z)
    }else{
        ((normal.x - normal.z) / SQRT_2, (normal.z + normal.x) / SQRT_2)
    };
    let (u, v) = if stereo_quincuncial {
        let mag = 1.0/(1.0+normal.y.abs());
        let _u = u0 * mag;
        let _v = v0 * mag;
        let u = 2.0 * _u.atan2(1.0 + _v.abs()) / PI;
        let v = 2.0 * _v.atan2(1.0 + _u.abs()) / PI;
        (u,v)
    }else{
        let mag = 1.0/(1.0+normal.y.abs()) + normal.y.abs()*0.5;
        let u = mag * u0.asin() / PI;
        let v = mag * v0.asin() / PI;
        (u,v)
    };
    if normal.y < 0.0 {
        [2.0-(0.5+u+v),0.5+v-u]
    }else{
        [0.5+u+v,0.5+v-u]
    }
}