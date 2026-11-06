struct Globals { screen: vec2<f32>, pad: vec2<f32> }

@group(0) @binding(0) var<uniform> g: Globals;
@group(0) @binding(1) var index_tex: texture_2d<f32>;
@group(0) @binding(2) var rgba_tex: texture_2d<f32>;
@group(0) @binding(3) var palette: texture_2d<f32>;
@group(0) @binding(4) var samp: sampler;

struct VIn {
    @location(0) pos: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) pal: u32,
    @location(3) mul: vec4<f32>,
    @location(4) add: vec4<f32>,
    @location(5) hue: f32,
}

struct VOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) pal: u32,
    @location(2) mul: vec4<f32>,
    @location(3) add: vec4<f32>,
    @location(4) @interpolate(flat) hue: f32,
}

@vertex
fn vs_main(v: VIn) -> VOut {
    var o: VOut;
    o.clip = vec4<f32>(v.pos.x / g.screen.x * 2.0 - 1.0, 1.0 - v.pos.y / g.screen.y * 2.0, 0.0, 1.0);
    o.uv = v.uv;
    o.pal = v.pal;
    o.mul = v.mul;
    o.add = v.add;
    o.hue = v.hue;
    return o;
}

@fragment
fn fs_main(v: VOut) -> @location(0) vec4<f32> {
    let mul = vec4<f32>(v.mul.rgb * 2.0, v.mul.a);
    let add = vec4<f32>(v.add.rgb, v.add.w * 3.0);
    let direct = textureSample(rgba_tex, samp, v.uv);
    var c = direct;
    if v.pal != 0xffffffffu {
        let dims = vec2<f32>(textureDimensions(index_tex));
        let texel = vec2<i32>(floor(v.uv * dims));
        let idx = i32(textureLoad(index_tex, texel, 0).r * 255.0 + 0.5);
        if idx == 0 {
            discard;
        }
        c = textureLoad(palette, vec2<i32>(idx, i32(v.pal)), 0);
    }
    let inv = add.w >= 1.5;
    let desat = add.w - select(0.0, 2.0, inv);
    var src = c.rgb;
    if inv {
        src = vec3<f32>(1.0) - src;
    }
    src = mix(src, vec3<f32>((src.r + src.g + src.b) / 3.0), desat);
    let ang = radians(v.hue);
    if abs(ang) > 0.0001 {
        let k = vec3<f32>(0.57735027);
        let ca = cos(ang);
        let sa = sin(ang);
        src = src * ca + cross(k, src) * sa + k * dot(k, src) * (1.0 - ca);
    }
    let rgb = clamp(src * mul.rgb + add.rgb, vec3<f32>(0.0), vec3<f32>(1.0));
    let a = c.a * mul.a;
    return vec4<f32>(rgb * a, a);
}
