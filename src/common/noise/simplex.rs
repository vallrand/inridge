use bevy::math::{Vec2,Vec3A,Vec4,IVec2,IVec3,Mat2,Mat3A};
use super::sampler::HeightSampler;
use super::perlin::{gradient_dot_2d,gradient_dot_3d};

///https://github.com/stegu/psrdnoise
#[derive(Default)]
pub struct SimplexNoise<T: Default> {
    period: T
}

impl HeightSampler<Vec2> for SimplexNoise<Vec2> {
    fn sample(&self, coordinate: Vec2, seed: u32) -> f32 {
        const M: Mat2 = Mat2::from_cols_array(&[1.0,0.5,0.0,1.0]);
        const M_INV: Mat2 = Mat2::from_cols_array(&[1.0,-0.5,0.0,1.0]);

        let uv = M * coordinate;
        
        let mut i0 = uv.floor().as_ivec2(); let f0 = uv.fract();
        let mut i1 = i0 + if f0.x < f0.y { IVec2::Y }else{ IVec2::X };
        let mut i2 = i0 + 1;

        let mut v0 = M_INV * i0.as_vec2();
        let mut v1 = M_INV * i1.as_vec2();
        let mut v2 = M_INV * i2.as_vec2();

        let d0 = coordinate - v0;
        let d1 = coordinate - v1;
        let d2 = coordinate - v2;

        if self.period.x > 0.0 {
            v0.x = v0.x.rem_euclid(self.period.x);
            v1.x = v1.x.rem_euclid(self.period.x);
            v2.x = v2.x.rem_euclid(self.period.x);
        }
        if self.period.y > 0.0 {
            v0.y = v0.y.rem_euclid(self.period.y);
            v1.y = v1.y.rem_euclid(self.period.y);
            v2.y = v2.y.rem_euclid(self.period.y);
        }
        i0 = (M * v0 + 0.5).floor().as_ivec2();
        i1 = (M * v1 + 0.5).floor().as_ivec2();
        i2 = (M * v2 + 0.5).floor().as_ivec2();

        let gradient = Vec3A::new(
            gradient_dot_2d(i0.x, i0.y, d0.x, d0.y, seed),
            gradient_dot_2d(i1.x, i1.y, d1.x, d1.y, seed),
            gradient_dot_2d(i2.x, i2.y, d2.x, d2.y, seed)
        );

        let mut w = 0.8 - Vec3A::new(
            Vec2::dot(d0, d0),
            Vec2::dot(d1, d1),
            Vec2::dot(d2, d2)
        );
        w = w.max(Vec3A::ZERO);
        w = w * w; w = w * w;
        10.9*Vec3A::dot(w, gradient)
    }
}

impl HeightSampler<Vec3A> for SimplexNoise<Vec3A> {
    fn sample(&self, coordinate: Vec3A, seed: u32) -> f32 {
        const M: Mat3A = Mat3A::from_cols_array(&[0.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 0.0]);
        const M_INV: Mat3A = Mat3A::from_cols_array(&[-0.5, 0.5, 0.5, 0.5,-0.5, 0.5, 0.5, 0.5,-0.5]);
        let uvw = M * coordinate;
        let mut i0 = uvw.floor().as_ivec3(); let f0 = uvw.fract();
        let g_ = IVec3::new(
            (f0.y >= f0.x) as i32,
            (f0.z >= f0.y) as i32,
            (f0.z >= f0.x) as i32,
        );
        let l_ = 1 - g_;
        let g = IVec3::new(l_.z, g_.x, g_.y);
        let l = IVec3::new(l_.x, l_.y, g_.z);

        let mut i1 = i0 + IVec3::min( g, l );
        let mut i2 = i0 + IVec3::max( g, l );
        let mut i3 = i0 + IVec3::ONE;

        let mut v0 = M_INV * i0.as_vec3a();
        let mut v1 = M_INV * i1.as_vec3a();
        let mut v2 = M_INV * i2.as_vec3a();
        let mut v3 = M_INV * i3.as_vec3a();

        let d0 = coordinate - v0;
        let d1 = coordinate - v1;
        let d2 = coordinate - v2;
        let d3 = coordinate - v3;
        
        if self.period.x > 0.0 {
            v0.x = v0.x.rem_euclid(self.period.x);
            v1.x = v1.x.rem_euclid(self.period.x);
            v2.x = v2.x.rem_euclid(self.period.x);
            v3.x = v3.x.rem_euclid(self.period.x);
        }
        if self.period.y > 0.0 {
            v0.y = v0.y.rem_euclid(self.period.y);
            v1.y = v1.y.rem_euclid(self.period.y);
            v2.y = v2.y.rem_euclid(self.period.y);
            v3.y = v3.y.rem_euclid(self.period.y);
        }
        if self.period.z > 0.0 {
            v0.z = v0.z.rem_euclid(self.period.z);
            v1.z = v1.z.rem_euclid(self.period.z);
            v2.z = v2.z.rem_euclid(self.period.z);
            v3.z = v3.z.rem_euclid(self.period.z);
        }
        
        i0 = (M * v0 + 0.5).floor().as_ivec3();
        i1 = (M * v1 + 0.5).floor().as_ivec3();
        i2 = (M * v2 + 0.5).floor().as_ivec3();
        i3 = (M * v3 + 0.5).floor().as_ivec3();
        
        let gradient = Vec4::new(
            gradient_dot_3d(i0.x, i0.y, i0.z, d0.x, d0.y, d0.z, seed),
            gradient_dot_3d(i1.x, i1.y, i1.z, d1.x, d1.y, d1.z, seed),
            gradient_dot_3d(i2.x, i2.y, i2.z, d2.x, d2.y, d2.z, seed),
            gradient_dot_3d(i3.x, i3.y, i3.z, d3.x, d3.y, d3.z, seed),
        );

        let mut w = 0.5 - Vec4::new(
            Vec3A::dot(d0,d0),
            Vec3A::dot(d1,d1),
            Vec3A::dot(d2,d2),
            Vec3A::dot(d3,d3)
        );
        w = Vec4::max(w, Vec4::ZERO);

        return 39.5 * Vec4::dot(w*w*w, gradient);
    }
}