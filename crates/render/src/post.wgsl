struct Params { p0: vec4<f32>, p1: vec4<f32>, p2: vec4<f32>, p3: vec4<f32>, p4: vec4<f32> }

@group(0) @binding(0) var<uniform> u: Params;
@group(0) @binding(1) var src: texture_2d<f32>;
@group(0) @binding(2) var bloom: texture_2d<f32>;
@group(0) @binding(3) var samp: sampler;

struct VOut { @builtin(position) clip: vec4<f32>, @location(0) uv: vec2<f32> }

@vertex
fn vs_full(@builtin(vertex_index) i: u32) -> VOut {
    let p = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u)) * 2.0 - 1.0;
    var o: VOut;
    o.clip = vec4<f32>(p, 0.0, 1.0);
    o.uv = vec2<f32>(p.x * 0.5 + 0.5, 0.5 - p.y * 0.5);
    return o;
}

@fragment
fn fs_bright(v: VOut) -> @location(0) vec4<f32> {
    let c = textureSample(src, samp, v.uv).rgb;
    let l = max(c.r, max(c.g, c.b));
    return vec4<f32>(c * smoothstep(0.58, 0.95, l) * 1.15, 1.0);
}

@fragment
fn fs_blur(v: VOut) -> @location(0) vec4<f32> {
    let d = u.p0.xy;
    var s = textureSample(src, samp, v.uv).rgb * 0.2270270270;
    s += (textureSample(src, samp, v.uv + d * 1.3846153846).rgb + textureSample(src, samp, v.uv - d * 1.3846153846).rgb) * 0.3162162162;
    s += (textureSample(src, samp, v.uv + d * 3.2307692308).rgb + textureSample(src, samp, v.uv - d * 3.2307692308).rgb) * 0.0702702703;
    return vec4<f32>(s, 1.0);
}

fn hash(p: vec2<f32>) -> f32 { return fract(sin(dot(p, vec2<f32>(12.9898, 78.233))) * 43758.5453); }

@fragment
fn fs_composite(v: VOut) -> @location(0) vec4<f32> {
    let c = v.uv - u.p2.yz;
    let r = length(c * vec2<f32>(u.p2.w, 1.0));
    let dir = c / max(length(c), 0.0001);
    let ca = u.p0.x * 0.0045 * (0.25 + r);
    var acc = vec3<f32>(0.0);
    for (var i = 0; i < 7; i++) {
        let p = v.uv - c * u.p2.x * 0.10 * (f32(i) / 6.0);
        acc += vec3<f32>(textureSample(src, samp, p + dir * ca).r, textureSample(src, samp, p).g, textureSample(src, samp, p - dir * ca).b);
    }
    var col = acc / 7.0;
    col += textureSample(bloom, samp, v.uv).rgb * u.p0.z;
    let luma = dot(col, vec3<f32>(0.299, 0.587, 0.114));
    col = mix(vec3<f32>(luma), col, u.p1.x);
    col = (col - 0.5) * u.p1.y + 0.5;
    col *= u.p3.xyz;
    col *= 1.0 - smoothstep(0.30, 0.95, r) * u.p0.y;
    col += vec3<f32>(u.p1.z);
    let l2 = dot(col, vec3<f32>(0.299, 0.587, 0.114));
    col = select(col, mix(vec3<f32>(1.0) - col, vec3<f32>(smoothstep(0.35, 0.65, 1.0 - l2)), 0.55), u.p4.x > 0.5);
    col += (hash(v.uv * 1024.0 + u.p1.w) - 0.5) * u.p0.w;
    return vec4<f32>(col, 1.0);
}
