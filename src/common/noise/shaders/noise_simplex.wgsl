#define_import_path bevy_noise::simplex

fn mod289v2f(x: vec2<f32>) -> vec2<f32> { return x - floor(x / 289.0) * 289.0; }
fn mod289v3f(x: vec3<f32>) -> vec3<f32> { return x - floor(x / 289.0) * 289.0; }
fn mod289v4f(x: vec4<f32>) -> vec4<f32> { return x - floor(x / 289.0) * 289.0; }
fn permute289v3f(x: vec3<f32>) -> vec3<f32> { return mod289v3f((x*34.0 + 10.0)*x); }
fn permute289v4f(x: vec4<f32>) -> vec4<f32> { return mod289v4f((x*34.0 + 10.0)*x); }

struct GradientNoise2D {
    noise: f32,
    gradient: vec2<f32>,
};
struct GradientNoise3D {
    noise: f32,
    gradient: vec3<f32>,
}

fn wrapped_simplex_noise2(x: vec2<f32>, period: vec2<f32>, alpha: f32) -> GradientNoise2D {	
	var uv = vec2<f32>(x.x+x.y*0.5, x.y);
	var i0 = floor(uv);
	var f0 = uv - i0;
	var o1 = select(vec2<f32>(0.0,1.0), vec2<f32>(1.0, 0.0), f0.x > f0.y);
	var i1 = i0 + o1;
	var i2 = i0 + vec2<f32>(1.0, 1.0);
	var v0 = vec2<f32>(i0.x - i0.y*0.5, i0.y);
	var v1 = vec2<f32>(v0.x + o1.x - o1.y*0.5, v0.y + o1.y);
	var v2 = vec2<f32>(v0.x + 0.5, v0.y + 1.0);
	var x0 = x - v0;
	var x1 = x - v1;
	var x2 = x - v2;

	var iu: vec3<f32>;
	var iv: vec3<f32>;
	var xw: vec3<f32>;
	var yw: vec3<f32>;

	if(any(period > vec2<f32>(0.0, 0.0))){
		xw = vec3<f32>(v0.x, v1.x, v2.x);
		yw = vec3<f32>(v0.y, v1.y, v2.y);
		if(period.x > 0.0) {
			xw = xw - floor(vec3<f32>(v0.x, v1.x, v2.x) / period.x) * period.x;
		}
		if(period.y > 0.0) {
			yw = yw - floor(vec3<f32>(v0.y, v1.y, v2.y) / period.y) * period.y;
		}
	iu = floor(xw + 0.5*yw + 0.5);
	iv = floor(yw + 0.5);
	} else {
		iu = vec3<f32>(i0.x, i1.x, i2.x);
		iv = vec3<f32>(i0.y, i1.y, i2.y);
	}

	var hash: vec3<f32> = mod289v3f(iu);
	hash = mod289v3f((hash*51.0 + 2.0)*hash + iv);
	hash = mod289v3f((hash*34.0 + 10.0)*hash);
	var psi: vec3<f32> = hash*0.07482 + alpha;
	var gx: vec3<f32> = cos(psi);
	var gy: vec3<f32> = sin(psi);
	var g0: vec2<f32> = vec2<f32>(gx.x, gy.x);
	var g1: vec2<f32> = vec2<f32>(gx.y, gy.y);
	var g2: vec2<f32> = vec2<f32>(gx.z, gy.z);

	var w: vec3<f32> = 0.8 - vec3<f32>(dot(x0, x0), dot(x1, x1), dot(x2, x2));
	w = max(w, vec3<f32>(0.0, 0.0, 0.0));
	var w2: vec3<f32> = w*w;
	var w4: vec3<f32> = w2*w2;
	var gdotx: vec3<f32> = vec3<f32>(dot(g0, x0), dot(g1, x1), dot(g2, x2));
	var n: f32 = 10.9*dot(w4, gdotx);

	var w3: vec3<f32> = w2*w;
	var dw: vec3<f32> = -8.0*w3*gdotx;
	var dn0: vec2<f32> = w4.x*g0 + dw.x*x0;
	var dn1: vec2<f32> = w4.y*g1 + dw.y*x1;
	var dn2: vec2<f32> = w4.z*g2 + dw.z*x2;
	var g: vec2<f32> = 10.9*(dn0 + dn1 + dn2);

	return GradientNoise2D(n, g);
}

fn wrapped_simplex_noise3(x: vec3<f32>, period: vec3<f32>, alpha: f32) -> GradientNoise3D {
	let M = mat3x3<f32>(0.0, 1.0, 1.0, 1.0, 0.0, 1.0,  1.0, 1.0, 0.0);
	let Mi = mat3x3<f32>(-0.5, 0.5, 0.5, 0.5,-0.5, 0.5, 0.5, 0.5,-0.5);
	
	var uvw: vec3<f32> = M * x;
	var i0: vec3<f32> = floor(uvw);
	var f0: vec3<f32> = uvw - i0;
	var gt_: vec3<f32> = step(f0.xyx, f0.yzz);
	var lt_: vec3<f32> = 1.0 - gt_;
	var gt: vec3<f32> = vec3<f32>(lt_.z, gt_.xy);
	var lt: vec3<f32> = vec3<f32>(lt_.xy, gt_.z);
	var o1: vec3<f32> = min( gt, lt );
	var o2: vec3<f32> = max( gt, lt );
	var i1: vec3<f32> = i0 + o1;
	var i2: vec3<f32> = i0 + o2;
	var i3: vec3<f32> = i0 + vec3<f32>(1.0,1.0,1.0);
	var v0: vec3<f32> = Mi * i0;
	var v1: vec3<f32> = Mi * i1;
	var v2: vec3<f32> = Mi * i2;
	var v3: vec3<f32> = Mi * i3;
	var x0: vec3<f32> = x - v0;
	var x1: vec3<f32> = x - v1;
	var x2: vec3<f32> = x - v2;
	var x3: vec3<f32> = x - v3;
	
	var vx: vec4<f32>;
	var vy: vec4<f32>;
	var vz: vec4<f32>;

	if(any(period > vec3<f32>(0.0))) {
		vx = vec4<f32>(v0.x, v1.x, v2.x, v3.x);
		vy = vec4<f32>(v0.y, v1.y, v2.y, v3.y);
		vz = vec4<f32>(v0.z, v1.z, v2.z, v3.z);
		if(period.x > 0.0) {
			vx = vx - floor(vx / period.x) * period.x;
		}
		if(period.y > 0.0) {
			vy = vy - floor(vy / period.y) * period.y;
		}
		if(period.z > 0.0) {
			vz = vz - floor(vz / period.z) * period.z;
		}
		i0 = floor(M * vec3<f32>(vx.x, vy.x, vz.x) + 0.5);
		i1 = floor(M * vec3<f32>(vx.y, vy.y, vz.y) + 0.5);
		i2 = floor(M * vec3<f32>(vx.z, vy.z, vz.z) + 0.5);
		i3 = floor(M * vec3<f32>(vx.w, vy.w, vz.w) + 0.5);
	}

	var hash: vec4<f32> = permute289v4f( permute289v4f( permute289v4f( 
		vec4<f32>(i0.z, i1.z, i2.z, i3.z ))
		+ vec4<f32>(i0.y, i1.y, i2.y, i3.y ))
		+ vec4<f32>(i0.x, i1.x, i2.x, i3.x ));
	var theta: vec4<f32> = hash * 3.883222077;
	var sz: vec4<f32> = hash * -0.006920415 + 0.996539792;
	var psi: vec4<f32> = hash * 0.108705628;
	var Ct: vec4<f32> = cos(theta);
	var St: vec4<f32> = sin(theta);
	var sz_: vec4<f32> = sqrt( 1.0 - sz*sz );

	var gx: vec4<f32>;
	var gy: vec4<f32>;
	var gz: vec4<f32>;
	var px: vec4<f32>;
	var py: vec4<f32>;
	var pz: vec4<f32>;
	var Sp: vec4<f32>;
	var Cp: vec4<f32>;
	var Ctp: vec4<f32>;
	var qx: vec4<f32>;
	var qy: vec4<f32>;
	var qz: vec4<f32>;
	var Sa: vec4<f32>;
	var Ca: vec4<f32>;

	if(alpha != 0.0){
		px = Ct * sz_;
		py = St * sz_;
		pz = sz;
		Sp = sin(psi);
		Cp = cos(psi);
		Ctp = St*Sp - Ct*Cp;
		qx = mix( Ctp*St, Sp, sz);
		qy = mix(-Ctp*Ct, Cp, sz);
		qz = -(py*Cp + px*Sp);
		Sa = vec4<f32>(sin(alpha));
		Ca = vec4<f32>(cos(alpha));
		gx = Ca*px + Sa*qx;
		gy = Ca*py + Sa*qy;
		gz = Ca*pz + Sa*qz;
	}else{
		gx = Ct * sz_;
		gy = St * sz_;
		gz = sz;  
	}
	
	var g0: vec3<f32> = vec3<f32>(gx.x, gy.x, gz.x);
	var g1: vec3<f32> = vec3<f32>(gx.y, gy.y, gz.y);
	var g2: vec3<f32> = vec3<f32>(gx.z, gy.z, gz.z);
	var g3: vec3<f32> = vec3<f32>(gx.w, gy.w, gz.w);
	var w: vec4<f32> = 0.5 - vec4<f32>(dot(x0,x0), dot(x1,x1), dot(x2,x2), dot(x3,x3));
	w = max(w, vec4<f32>(0.0, 0.0, 0.0, 0.0));
	var w2: vec4<f32> = w * w;
	var w3: vec4<f32> = w2 * w;
	var gdotx: vec4<f32> = vec4<f32>(dot(g0,x0), dot(g1,x1), dot(g2,x2), dot(g3,x3));
	var n: f32 = 39.5 * dot(w3, gdotx);

	var dw: vec4<f32> = -6.0 * w2 * gdotx;
	var dn0: vec3<f32> = w3.x * g0 + dw.x * x0;
	var dn1: vec3<f32> = w3.y * g1 + dw.y * x1;
	var dn2: vec3<f32> = w3.z * g2 + dw.z * x2;
	var dn3: vec3<f32> = w3.w * g3 + dw.w * x3;
	var g: vec3<f32> = 39.5 * (dn0 + dn1 + dn2 + dn3);
	
	return GradientNoise3D(n, g);
}

fn simplex_noise2(v: vec2<f32>) -> f32 {
    let C = vec4<f32>(0.211324865405187, 0.366025403784439, -0.577350269189626, 0.024390243902439);
    var i: vec2<f32> = floor(v + dot(v, C.yy));
    let x0 = v - i + dot(i, C.xx);
    var i1: vec2<f32> = select(vec2<f32>(0., 1.), vec2<f32>(1., 0.), (x0.x > x0.y));
    var x12: vec4<f32> = x0.xyxy + C.xxzz - vec4<f32>(i1, 0., 0.);

    i = mod289v2f(i);
    let p = permute289v3f(permute289v3f(i.y + vec3<f32>(0., i1.y, 1.)) + i.x + vec3<f32>(0., i1.x, 1.));

    var m: vec3<f32> = max(0.5 - vec3<f32>(dot(x0, x0), dot(x12.xy, x12.xy), dot(x12.zw, x12.zw)), vec3<f32>(0.));
    m = m * m;
    m = m * m;
    let x = 2. * fract(p * C.www) - 1.;
    let h = abs(x) - 0.5;
    let ox = floor(x + 0.5);
    let a0 = x - ox;
    m = m * (1.79284291400159 - 0.85373472095314 * (a0 * a0 + h * h));
    let g = vec3<f32>(a0.x * x0.x + h.x * x0.y, a0.yz * x12.xz + h.yz * x12.yw);

    return 130. * dot(m, g);
}

fn simplex_noise3(v: vec3<f32>) -> f32 {
    let C = vec2<f32>(1. / 6., 1. / 3.);
    let D = vec4<f32>(0., 0.5, 1., 2.);
    var i: vec3<f32>  = floor(v + dot(v, C.yyy));
    let x0 = v - i + dot(i, C.xxx);

    let g = step(x0.yzx, x0.xyz);
    let l = 1.0 - g;
    let i1 = min(g.xyz, l.zxy);
    let i2 = max(g.xyz, l.zxy);

    let x1 = x0 - i1 + 1. * C.xxx;
    let x2 = x0 - i2 + 2. * C.xxx;
    let x3 = x0 - 1. + 3. * C.xxx;

    i = mod289v3f(i);
    let p = permute289v4f(permute289v4f(permute289v4f(
    i.z + vec4<f32>(0., i1.z, i2.z, 1. )) +
    i.y + vec4<f32>(0., i1.y, i2.y, 1. )) +
    i.x + vec4<f32>(0., i1.x, i2.x, 1. ));

    var n_: f32 = 1. / 7.;
    let ns = n_ * D.wyz - D.xzx;

    let j = p - 49. * floor(p * ns.z * ns.z);

    let x_ = floor(j * ns.z);
    let y_ = floor(j - 7.0 * x_);

    let x = x_ *ns.x + ns.yyyy;
    let y = y_ *ns.x + ns.yyyy;
    let h = 1.0 - abs(x) - abs(y);

    let b0 = vec4<f32>(x.xy, y.xy);
    let b1 = vec4<f32>(x.zw, y.zw);

    let s0 = floor(b0)*2.0 + 1.0;
    let s1 = floor(b1)*2.0 + 1.0;
    let sh = -step(h, vec4<f32>(0.));

    let a0 = b0.xzyw + s0.xzyw*sh.xxyy;
    let a1 = b1.xzyw + s1.xzyw*sh.zzww;

    var p0: vec3<f32> = vec3<f32>(a0.xy, h.x);
    var p1: vec3<f32> = vec3<f32>(a0.zw, h.y);
    var p2: vec3<f32> = vec3<f32>(a1.xy, h.z);
    var p3: vec3<f32> = vec3<f32>(a1.zw, h.w);

    let norm = 1.79284291400159 - 0.85373472095314 * vec4<f32>(dot(p0,p0), dot(p1,p1), dot(p2,p2), dot(p3,p3));
    p0 = p0 * norm.x;
    p1 = p1 * norm.y;
    p2 = p2 * norm.z;
    p3 = p3 * norm.w;

    var m: vec4<f32> = max(0.5 - vec4<f32>(dot(x0,x0), dot(x1,x1), dot(x2,x2), dot(x3,x3)), vec4<f32>(0.));
    m = m * m;
    m = m * m;
    let pdotx = vec4<f32>(dot(p0,x0), dot(p1,x1), dot(p2,x2), dot(p3,x3));

    return 105.0 * dot(m, pdotx);
}