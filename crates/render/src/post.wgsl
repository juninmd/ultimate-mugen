struct Params { p0: vec4<f32>, p1: vec4<f32>, p2: vec4<f32>, p3: vec4<f32>, p4: vec4<f32>, p5: vec4<f32>, p6: vec4<f32>, p7: vec4<f32>, p8: vec4<f32>, waves: array<vec4<f32>, 8>, glass: array<vec4<f32>, 32> }

@group(0) @binding(0) var<uniform> u: Params;
@group(0) @binding(1) var src: texture_2d<f32>;
@group(0) @binding(2) var bloom: texture_2d<f32>;
@group(0) @binding(3) var samp: sampler;
@group(0) @binding(4) var frost: texture_2d<f32>;

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
    return vec4<f32>(c * smoothstep(u.p6.y, u.p6.y + 0.37, l) * 1.15, 1.0);
}

@fragment
fn fs_blur(v: VOut) -> @location(0) vec4<f32> {
    let d = u.p0.xy;
    var s = textureSample(src, samp, v.uv).rgb * 0.2270270270;
    s += (textureSample(src, samp, v.uv + d * 1.3846153846).rgb + textureSample(src, samp, v.uv - d * 1.3846153846).rgb) * 0.3162162162;
    s += (textureSample(src, samp, v.uv + d * 3.2307692308).rgb + textureSample(src, samp, v.uv - d * 3.2307692308).rgb) * 0.0702702703;
    return vec4<f32>(s, 1.0);
}

@fragment
fn fs_copy(v: VOut) -> @location(0) vec4<f32> {
    let d = u.p0.zw;
    let s = textureSample(src, samp, v.uv + vec2<f32>(d.x, d.y)).rgb + textureSample(src, samp, v.uv + vec2<f32>(-d.x, d.y)).rgb + textureSample(src, samp, v.uv + vec2<f32>(d.x, -d.y)).rgb + textureSample(src, samp, v.uv + vec2<f32>(-d.x, -d.y)).rgb;
    return vec4<f32>(s * 0.25, 1.0);
}

fn rounded_box(p: vec2<f32>, c: vec2<f32>, h: vec2<f32>, r: f32) -> f32 {
    let q = abs(p - c) - h + vec2<f32>(r);
    return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - r;
}

fn glass_layer(base: vec3<f32>, uv: vec2<f32>, blurred: vec3<f32>) -> vec3<f32> {
    let px = uv * u.p4.yz;
    let th = u.p4.w;
    let luma = dot(blurred, vec3<f32>(0.299, 0.587, 0.114));
    let frosted = mix(vec3<f32>(luma), blurred, 1.3);
    var col = base;
    let n = i32(u.p3.w);
    for (var i = 0; i < n; i++) {
        let a = u.glass[2 * i];
        let b = u.glass[2 * i + 1];
        let c = (a.xy + a.zw) * 0.5;
        let h = (a.zw - a.xy) * 0.5;
        let d = rounded_box(px, c, h, b.x);
        let reach = b.x * 1.2 + 9.0 * th;
        let ds = rounded_box(px, c + vec2<f32>(0.0, 3.0 * th), h, b.x);
        let shadow = (1.0 - smoothstep(-reach * 0.15, reach, ds)) * 0.42 * step(0.0, d);
        col = col * (1.0 - shadow);
        let inside = 1.0 - smoothstep(-0.75, 0.75, d);
        let ty = clamp((px.y - a.y) / max(a.w - a.y, 1.0), 0.0, 1.0);
        let tx = clamp((px.x - a.x) / max(a.z - a.x, 1.0), 0.0, 1.0);
        var fr = frosted * 0.6 + b.yzw * 0.17 * (1.0 - ty * 0.5) + vec3<f32>(0.02);
        fr *= mix(1.07, 0.88, ty);
        let streak = 1.0 - smoothstep(0.0, 0.18, abs(tx - 0.22 - ty * 0.45));
        fr += vec3<f32>(0.06) * streak * (1.0 - ty * 0.6);
        col = mix(col, fr, inside * 0.94);
        let rim = 1.0 - smoothstep(0.0, 1.3 * th, abs(d + 0.7 * th));
        let rim_c = mix(vec3<f32>(0.95, 0.97, 1.0), vec3<f32>(0.55, 0.62, 0.8), pow(ty, 0.7));
        col = mix(col, rim_c, rim * mix(0.6, 0.22, ty));
    }
    return col;
}

fn hash(p: vec2<f32>) -> f32 { return fract(sin(dot(p, vec2<f32>(12.9898, 78.233))) * 43758.5453); }

fn aces(x: vec3<f32>) -> vec3<f32> {
    return clamp((x * (2.51 * x + 0.03)) / (x * (2.43 * x + 0.59) + 0.14), vec3<f32>(0.0), vec3<f32>(1.0));
}

fn distortion(uv: vec2<f32>) -> vec2<f32> {
    let size = u.p4.yz;
    let px = uv * size;
    var off = vec2<f32>(0.0);
    let n = i32(u.p7.w);
    for (var i = 0; i < n; i++) {
        let a = u.waves[2 * i];
        let b = u.waves[2 * i + 1];
        let d = px - a.xy;
        let len = max(length(d), 0.001);
        let k = (len - a.z) / a.w;
        off += d / len * exp(-k * k) * b.x * a.w * 0.5 / size;
    }
    let heat = u.p5.w;
    if (heat > 0.0) {
        let mask = 1.0 - smoothstep(0.08, 0.75, length((uv - u.p2.yz) * vec2<f32>(u.p2.w, 1.0)));
        let ph = u.p1.w * 0.11;
        off += vec2<f32>(sin(uv.y * 70.0 + ph) + 0.5 * sin(uv.y * 150.0 - ph * 1.7), cos(uv.x * 55.0 + ph * 0.8)) * heat * mask * 0.0014;
    }
    return off;
}

fn grade_color(col: vec3<f32>) -> vec3<f32> {
    let lit = col * u.p5.y;
    var c = mix(lit, aces(lit), u.p5.x);
    let l = dot(c, vec3<f32>(0.299, 0.587, 0.114));
    let tone = mix(u.p7.xyz, u.p8.xyz, smoothstep(0.12, 0.85, l));
    return c * mix(vec3<f32>(1.0), tone, u.p5.z);
}

@fragment
fn fs_composite(v: VOut) -> @location(0) vec4<f32> {
    let uvd = v.uv + distortion(v.uv);
    let c = v.uv - u.p2.yz;
    let r = length(c * vec2<f32>(u.p2.w, 1.0));
    let dir = c / max(length(c), 0.0001);
    let ca = u.p0.x * 0.0045 * (0.25 + r);
    var acc = vec3<f32>(0.0);
    for (var i = 0; i < 7; i++) {
        let p = uvd - c * u.p2.x * 0.10 * (f32(i) / 6.0);
        acc += vec3<f32>(textureSample(src, samp, p + dir * ca).r, textureSample(src, samp, p).g, textureSample(src, samp, p - dir * ca).b);
    }
    var col = acc / 7.0;
    col += textureSample(bloom, samp, uvd).rgb * u.p0.z;
    col = grade_color(col);
    let luma = dot(col, vec3<f32>(0.299, 0.587, 0.114));
    col = mix(vec3<f32>(luma), col, u.p1.x);
    col = (col - 0.5) * u.p1.y + 0.5;
    col *= u.p3.xyz;
    col *= 1.0 - smoothstep(0.30, 0.95, r) * u.p0.y;
    col += vec3<f32>(u.p1.z);
    let l2 = dot(col, vec3<f32>(0.299, 0.587, 0.114));
    col = select(col, mix(vec3<f32>(1.0) - col, vec3<f32>(smoothstep(0.35, 0.65, 1.0 - l2)), 0.55), u.p4.x > 0.5);
    col += (hash(v.uv * 1024.0 + u.p1.w) - 0.5) * u.p0.w;
    let blurred = textureSample(frost, samp, v.uv).rgb;
    if (u.p3.w > 0.5) { col = glass_layer(col, v.uv, blurred); }
    return vec4<f32>(col, 1.0);
}
