//! Face, hair and head-gear detail (see `detail.rs`). Face code works in head-local units (1 = head radius).
use super::detail::*;
use super::*;

/// lower jaw outline y(x) of the head polygon, used to keep stubble inside the face
fn jaw_y(x: f64) -> f64 {
    let pts = [(-0.3, 0.74), (0.2, 1.0), (0.62, 0.98), (0.93, 0.48)];
    for w in pts.windows(2) {
        let ((x0, y0), (x1, y1)) = (w[0], w[1]);
        if x >= x0 && x <= x1 { return y0 + (y1 - y0) * (x - x0) / (x1 - x0); }
    }
    0.5
}

fn thin(mut s: Shape) -> Shape { s.no_outline = true; s.line = true; s }

impl<'a> Rig<'a> {
    fn beard_col(&self) -> Rgb { self.c.get("beard").unwrap_or_else(|| mix(self.c.c("hair"), [0, 0, 0], 0.12)) }
    fn beard_list(&self) -> Vec<String> {
        match gv(self.style, "beard") {
            Some(Value::String(s)) => vec![s.clone()],
            Some(Value::Array(a)) => a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect(),
            _ => vec![],
        }
    }

    /// shading + facial hair, drawn before the eyes / mouth (head-local coordinates)
    pub(super) fn face_pre(&self, ctx: &mut Ctx) {
        if self.helmet { return; }
        let skin = self.c.c("skin");
        let (hi, shd) = (light_of(skin), shade_of(skin));
        let fs = self.sn("faceShade", 0.0).min(1.5);
        let ls = |ctx: &mut Ctx, l: Vec<Vec<P>>, c: Rgb, a: f64, w: f64| stroke_lines(ctx, &l, c, a, w);
        if fs > 0.0 {
            // nose side + bridge, cheekbone hollow, jaw line, eye bag
            fill_poly(ctx, &[pt(0.9, 0.16), pt(1.0, 0.4), pt(0.9, 0.47), pt(0.82, 0.42)], shd, 0.22 * fs, true);
            ls(ctx, vec![vec![pt(0.9, 0.06), pt(0.98, 0.3)]], hi, 0.45 * fs, 0.05);
            ls(ctx, vec![qc(pt(0.1, 0.5), pt(0.45, 0.62), pt(0.8, 0.54), 6)], shd, 0.2 * fs, 0.12);
            ls(ctx, vec![qc(pt(-0.05, 0.72), pt(0.3, 0.97), pt(0.88, 0.62), 8)], shd, 0.3 * fs, 0.09);
            ls(ctx, vec![qc(pt(0.18, 0.36), pt(0.38, 0.44), pt(0.58, 0.34), 5)], shd, 0.24 * fs, 0.045);
            ls(ctx, vec![qc(pt(0.62, 0.34), pt(0.8, 0.38), pt(0.92, 0.33), 4)], shd, 0.18 * fs, 0.04);
            dots(ctx, &[pt(0.96, 0.4)], 0.035, WHITE, 0.45 * fs);
            // forehead / temple highlight, chin dimple
            fill_ell(ctx, pt(0.55, -0.3), 0.22, 0.09, -0.3, hi, 0.2 * fs);
            ls(ctx, vec![vec![pt(0.62, 0.9), pt(0.68, 0.82)]], shd, 0.25 * fs, 0.035);
        }
        let bc = self.beard_col();
        let ol = self.c.c("outline");
        for kind in self.beard_list() {
            match kind.as_str() {
                "stubble" => {
                    fill_poly(ctx, &[pt(-0.2, 0.42), pt(-0.3, 0.74), pt(0.2, 1.0), pt(0.62, 0.98), pt(0.93, 0.48), pt(0.7, 0.58), pt(0.35, 0.56), pt(0.0, 0.45)], bc, 0.13, true);
                    let mut ps = vec![];
                    for i in 0..110 {
                        let x = -0.25 + hash(i as f64 * 1.7) * 1.2;
                        let y = 0.48 + hash(i as f64 * 3.3 + 1.0) * 0.52;
                        if y < jaw_y(x) - 0.05 && !(y > 0.6 && y < 0.76 && x > 0.5 && x < 0.86) && !(x > 0.75 && y < 0.56) { ps.push(pt(x, y)); }
                    }
                    dots(ctx, &ps, 0.017, bc, 0.55);
                }
                "goatee" => {
                    let pts = [pt(0.42, 0.74), pt(0.78, 0.72), pt(0.84, 0.9), pt(0.66, 1.14), pt(0.46, 1.16), pt(0.3, 0.98), pt(0.3, 0.84)];
                    fill_poly(ctx, &pts, bc, 1.0, true);
                    stroke_poly(ctx, &pts, ol, 0.8, 0.03, true);
                    ls(ctx, (0..4).map(|i| vec![pt(0.45 + i as f64 * 0.1, 0.8), pt(0.5 + i as f64 * 0.09, 1.08)]).collect(), light_of(bc), 0.35, 0.025);
                }
                "full" => {
                    let pts = [pt(-0.2, 0.36), pt(-0.34, 0.8), pt(0.1, 1.1), pt(0.62, 1.16), pt(0.98, 0.66), pt(0.95, 0.5), pt(0.74, 0.55), pt(0.52, 0.6), pt(0.3, 0.55), pt(0.1, 0.42)];
                    fill_poly(ctx, &pts, bc, 1.0, true);
                    stroke_poly(ctx, &pts, ol, 0.8, 0.03, true);
                    // mouth gap + strands
                    fill_poly(ctx, &[pt(0.5, 0.64), pt(0.85, 0.62), pt(0.84, 0.74), pt(0.52, 0.76)], mix(self.c.c("skin"), bc, 0.3), 0.9, true);
                    ls(ctx, (0..7).map(|i| { let x = 0.0 + i as f64 * 0.12; vec![pt(x, 0.7), pt(x + 0.03, jaw_y(x.min(0.9)) + 0.1)] }).collect(), light_of(bc), 0.3, 0.025);
                    ls(ctx, vec![qc(pt(-0.1, 0.45), pt(0.0, 0.7), pt(0.2, 0.9), 5)], shade_of(bc), 0.6, 0.03);
                }
                "moustache" => {
                    let pts = [pt(0.5, 0.52), pt(0.78, 0.5), pt(1.0, 0.54), pt(0.99, 0.62), pt(0.84, 0.62), pt(0.66, 0.68), pt(0.52, 0.62)];
                    fill_poly(ctx, &pts, bc, 1.0, true);
                    stroke_poly(ctx, &pts, ol, 0.75, 0.028, true);
                    ls(ctx, vec![vec![pt(0.55, 0.55), pt(0.95, 0.56)]], light_of(bc), 0.3, 0.02);
                }
                "chinstrap" => {
                    ls(ctx, vec![qc(pt(-0.18, 0.42), pt(0.0, 0.95), pt(0.66, 1.0), 9)], bc, 1.0, 0.07);
                    ls(ctx, vec![qc(pt(0.66, 1.0), pt(0.86, 0.9), pt(0.94, 0.6), 5)], bc, 1.0, 0.06);
                }
                "sideburns" => fill_poly(ctx, &[pt(-0.2, 0.0), pt(-0.02, 0.0), pt(0.02, 0.52), pt(-0.12, 0.72), pt(-0.26, 0.5)], bc, 1.0, true),
                _ => {}
            }
        }
    }

    /// brows, scars, glasses, eye patch, paint, earring (drawn after eyes / mouth)
    pub(super) fn face_post(&self, ctx: &mut Ctx) {
        let (c, style) = (&self.c, self.style);
        let ol = c.c("outline");
        let skin = c.c("skin");
        let fem = self.sk.fem;
        let face = gstr(self.pose, "face").unwrap_or("neutral");
        if self.helmet { self.helmet_detail(ctx); return; }
        if let Some(b) = self.sv("brow") {
            let th = gnum(b, "thick", 1.6);
            let col = self.col_of(gv(b, "color"), mix(c.c("hair"), c.c("outline"), 0.35));
            let shape = gstr(b, "shape").unwrap_or("flat");
            let angry = face == "shout" || face == "angry" || face == "hurt";
            let tilt = match shape { "angry" => 0.2, "arched" => -0.08, _ => 0.0 } + if angry { 0.1 } else { 0.0 };
            let one = |cx: f64, w: f64, k: f64| -> Vec<P> {
                let (x0, x1) = (cx - w * 0.55, cx + w * 0.5);
                let (y0, y1) = (-0.14 - tilt * 0.3 * w, -0.1 + tilt * 0.8 * w);
                let ym = (y0 + y1) * 0.5 - if shape == "arched" { 0.07 * k } else { 0.0 };
                let t = 0.05 * th * k;
                vec![pt(x0, y0 - t * 0.6), pt((x0 + x1) * 0.5, ym - t), pt(x1, y1 - t * 0.5), pt(x1, y1 + t * 0.4), pt((x0 + x1) * 0.5, ym + t * 0.7), pt(x0, y0 + t * 0.6)]
            };
            for (cx, w, k) in [(0.36, 0.42, 1.0), (0.8, 0.22, 0.8)] { fill_poly(ctx, &one(cx, w, k), col, 1.0, true); }
            stroke_lines(ctx, &[vec![pt(0.16, -0.17), pt(0.5, -0.12 - 0.02)]], light_of(col), 0.25, 0.02);
        }
        if let Some(sc) = gv(style, "scars").and_then(|v| v.as_array()) {
            let scc = mix(skin, [0xa0, 0x30, 0x40], 0.75);
            for part in sc.iter().filter_map(|x| x.as_str()) {
                let (a, b) = match part { "brow" => (pt(0.3, -0.35), pt(0.5, 0.08)), "cheek" => (pt(0.35, 0.34), pt(0.82, 0.62)), "jaw" => (pt(0.1, 0.62), pt(0.5, 0.95)), "nose" => (pt(0.72, 0.2), pt(0.98, 0.34)), _ => continue };
                stroke_lines(ctx, &[vec![a, b]], scc, 0.95, 0.055);
                let n = norm(perp(sub(b, a)));
                let ticks: Vec<Vec<P>> = (1..4).map(|i| { let p = lerp(a, b, i as f64 / 4.0); vec![add(p, n, -0.06), add(p, n, 0.06)] }).collect();
                stroke_lines(ctx, &ticks, mix(scc, [0xf0, 0xe0, 0xd0], 0.5), 0.9, 0.022);
            }
        }
        if let Some(p) = gv(style, "facePaint").filter(|v| v.is_object()) {
            let pc = self.col_of(gv(p, "color"), c.c("top2"));
            let a = gnum(p, "alpha", 0.95);
            match gstr(p, "type").unwrap_or("bars") {
                "tears" => { for x in [0.34, 0.5] { fill_poly(ctx, &[pt(x, 0.3), pt(x + 0.07, 0.34), pt(x + 0.04, 0.78), pt(x - 0.02, 0.74)], pc, a, true); } }
                "eyeband" => fill_poly(ctx, &[pt(0.05, 0.02), pt(1.0, 0.0), pt(1.0, 0.3), pt(0.05, 0.34)], pc, a * 0.9, true),
                "tribal" => {
                    fill_poly(ctx, &[pt(0.15, 0.4), pt(0.6, 0.38), pt(0.88, 0.58), pt(0.5, 0.52)], pc, a, true);
                    fill_poly(ctx, &[pt(0.2, 0.5), pt(0.52, 0.56), pt(0.76, 0.84), pt(0.38, 0.66)], pc, a, true);
                    fill_poly(ctx, &[pt(0.46, -0.52), pt(0.58, -0.25), pt(0.52, -0.52)], pc, a, false);
                }
                "dot" => { dots(ctx, &[pt(0.96, -0.2)], 0.07, pc, a); }
                "lines" => stroke_lines(ctx, &[vec![pt(0.2, 0.4), pt(0.6, 0.38)], vec![pt(0.24, 0.5), pt(0.62, 0.48)], vec![pt(0.3, 0.6), pt(0.62, 0.58)]], pc, a, 0.07),
                "mask" => fill_poly(ctx, &[pt(0.1, 0.0), pt(1.0, -0.08), pt(1.0, 0.36), pt(0.7, 0.46), pt(0.4, 0.36), pt(0.1, 0.4)], pc, a * 0.85, true),
                "slash" => { stroke_lines(ctx, &[vec![pt(0.2, 0.3), pt(0.75, 0.7)], vec![pt(0.3, 0.26), pt(0.85, 0.66)]], pc, a, 0.06); }
                _ => stroke_lines(ctx, &[vec![pt(0.25, 0.42), pt(0.62, 0.4)], vec![pt(0.3, 0.52), pt(0.6, 0.5)]], pc, a, 0.07),
            }
        }
        if let Some(g) = self.sv("glasses") {
            let kind = gstr(g, "type").unwrap_or("round");
            let fc = self.col_of(gv(g, "color"), [0x22, 0x1e, 0x2a]);
            let tint = self.col_of(gv(g, "lens"), mix(WHITE, c.c("eye"), 0.5));
            match kind {
                "shades" => {
                    let pts = [pt(0.06, -0.02), pt(0.98, -0.04), pt(0.96, 0.24), pt(0.7, 0.32), pt(0.38, 0.36), pt(0.1, 0.28)];
                    fill_poly(ctx, &pts, fc, 0.96, true);
                    stroke_poly(ctx, &pts, [0, 0, 0], 0.8, 0.03, true);
                    stroke_lines(ctx, &[vec![pt(0.2, 0.02), pt(0.38, 0.12)], vec![pt(0.46, 0.0), pt(0.58, 0.07)]], WHITE, 0.55, 0.035);
                    stroke_lines(ctx, &[vec![pt(0.06, 0.04), pt(-0.18, 0.12)]], fc, 1.0, 0.045);
                }
                "rect" | "round" => {
                    let rr = if kind == "round" { 0.26 } else { 0.24 };
                    for (cx, cy, k) in [(0.36, 0.16, 1.0), (0.8, 0.15, 0.62)] {
                        let (rx, ry) = (rr * k * 1.05, rr * k * 0.95);
                        if kind == "round" {
                            fill_ell(ctx, pt(cx, cy), rx, ry, 0.0, tint, 0.22);
                            ctx.begin(); Geo::Ellipse(pt(cx, cy), rx, ry, 0.0).emit(ctx, 0.0, 0.0); ctx.stroke_w(&sol(fc, 1.0), 0.05 + 0.02 * k);
                            stroke_lines(ctx, &[qc(pt(cx - rx * 0.6, cy - ry * 0.3), pt(cx - rx * 0.3, cy - ry * 0.7), pt(cx + rx * 0.1, cy - ry * 0.75), 4)], WHITE, 0.6, 0.03);
                        } else {
                            let pts = [pt(cx - rx, cy - ry * 0.7), pt(cx + rx, cy - ry * 0.8), pt(cx + rx * 0.9, cy + ry * 0.8), pt(cx - rx, cy + ry * 0.7)];
                            fill_poly(ctx, &pts, tint, 0.22, false);
                            stroke_poly(ctx, &pts, fc, 1.0, 0.05 + 0.02 * k, false);
                            stroke_lines(ctx, &[vec![pt(cx - rx * 0.6, cy - ry * 0.4), pt(cx - rx * 0.1, cy - ry * 0.55)]], WHITE, 0.6, 0.03);
                        }
                    }
                    stroke_lines(ctx, &[qc(pt(0.58, 0.1), pt(0.65, 0.04), pt(0.7, 0.1), 4), vec![pt(0.1, 0.12), pt(-0.15, 0.2)]], fc, 1.0, 0.045);
                }
                "monocle" => {
                    ctx.begin(); Geo::Ellipse(pt(0.36, 0.16), 0.28, 0.27, 0.0).emit(ctx, 0.0, 0.0); ctx.stroke_w(&sol(self.c.c("accent"), 1.0), 0.05);
                    fill_ell(ctx, pt(0.36, 0.16), 0.27, 0.26, 0.0, tint, 0.15);
                    stroke_lines(ctx, &[qc(pt(0.2, 0.42), pt(0.1, 0.8), pt(-0.1, 1.0), 6)], self.c.c("accent"), 0.9, 0.025);
                }
                _ => {}
            }
        }
        if let Some(p) = self.sv("eyepatch") {
            let pc = self.col_of(Some(p), [0x18, 0x14, 0x1c]);
            stroke_lines(ctx, &[vec![pt(0.1, 0.0), pt(-0.5, -0.38)], vec![pt(0.6, 0.0), pt(0.7, -0.35)]], pc, 1.0, 0.035);
            fill_ell(ctx, pt(0.36, 0.17), 0.3, 0.27, -0.1, pc, 1.0);
            fill_ell(ctx, pt(0.3, 0.1), 0.14, 0.06, -0.5, light_of(pc), 0.4);
        }
        if let Some(e) = self.sv("earring") {
            let ec = self.col_of(Some(e), c.c("accent"));
            let p = pt(-0.12, 0.47);
            dots(ctx, &[p], 0.045, ec, 1.0);
            ctx.begin(); ctx.move_to(p.x + 0.09, p.y + 0.1); ctx.arc(p.x, p.y + 0.1, 0.09, 0.0, PI * 2.0); ctx.stroke_w(&sol(ec, 1.0), 0.035);
            dots(ctx, &[pt(p.x - 0.03, p.y + 0.05)], 0.015, WHITE, 0.8);
        }
        // lip + lower lip highlight for female faces
        if fem && self.sn("faceShade", 0.0) > 0.0 && !gt(style, "mask") && face != "shout" && face != "hurt" && face != "ko" {
            stroke_lines(ctx, &[vec![pt(0.58, 0.725), pt(0.76, 0.715)]], light_of(c.c("lips")), 0.6, 0.035);
            // lashes
            stroke_lines(ctx, &[vec![pt(0.12, 0.1), pt(0.02, 0.0)], vec![pt(0.14, 0.18), pt(0.04, 0.2)]], ol, 0.8, 0.03);
        }
    }

    /// android helmet: ear cup, cheek vents, panel seams, rivets
    fn helmet_detail(&self, ctx: &mut Ctx) {
        if self.sv("helmetDetail").is_none() { return; }
        let m = self.c.c("metal");
        let ac = self.c.c("accent");
        let (hi, shd) = (light_of(m), shade_of(m));
        // ear cup
        fill_ell(ctx, pt(-0.12, 0.28), 0.32, 0.36, 0.0, shd, 0.9);
        fill_ell(ctx, pt(-0.1, 0.27), 0.25, 0.28, 0.0, m, 1.0);
        fill_ell(ctx, pt(-0.1, 0.27), 0.12, 0.14, 0.0, ac, 1.0);
        stroke_lines(ctx, &[qc(pt(-0.3, 0.12), pt(-0.12, -0.02), pt(0.08, 0.1), 5)], hi, 0.7, 0.04);
        // panel seams + vents
        stroke_lines(ctx, &[qc(pt(-0.3, -0.35), pt(0.2, -0.5), pt(0.95, -0.3), 8), vec![pt(0.16, 0.34), pt(0.16, 0.75), pt(0.5, 0.98)], qc(pt(-0.55, 0.0), pt(-0.5, 0.45), pt(-0.28, 0.72), 5)], shd, 0.8, 0.035);
        stroke_lines(ctx, &(0..4).map(|i| vec![pt(0.42, 0.52 + i as f64 * 0.1), pt(0.8, 0.5 + i as f64 * 0.1)]).collect::<Vec<_>>(), shd, 0.85, 0.04);
        stroke_lines(ctx, &(0..4).map(|i| vec![pt(0.42, 0.55 + i as f64 * 0.1), pt(0.8, 0.53 + i as f64 * 0.1)]).collect::<Vec<_>>(), hi, 0.4, 0.02);
        dots(ctx, &[pt(-0.32, -0.3), pt(0.9, -0.28), pt(-0.5, 0.5), pt(0.2, 0.9)], 0.035, shd, 0.9);
        dots(ctx, &[pt(-0.31, -0.32), pt(0.91, -0.3)], 0.014, WHITE, 0.7);
    }

    // ------------------------------------------------------------------ hair
    /// strands / locks / gloss clipped to the hair outline (After::Hair)
    pub(super) fn hair_detail(&self, ctx: &mut Ctx, geo: &Geo) {
        let hair = &self.hair;
        let n = gnum(hair, "strands", 0.0) as i32;
        let locks = gnum(hair, "locks", 0.0) as i32;
        let shine = gt(hair, "shine");
        if n <= 0 && locks <= 0 && !shine { return; }
        let hcol = self.c.c("hair");
        let r = self.sk.r;
        let sweep = gnum(hair, "sweep", if self.ht == "spiky" { 28.0 } else { 40.0 });
        let a0 = match self.ht.as_str() { "mane" | "wild" => 115.0, "long" | "flowing" => 130.0, _ => 150.0 };
        let a1 = gnum(hair, "front", 318.0);
        ctx.save();
        clip_geo(ctx, std::slice::from_ref(geo));
        let strand = |k: f64, t: f64, off: f64| -> Vec<P> {
            let a = (a0 + (a1 - a0) * t + off) * D2R;
            let b = (a0 + (a1 - a0) * t - sweep * 0.45 + off) * D2R;
            let e = (a0 + (a1 - a0) * t - sweep * 0.95 + off) * D2R;
            let rad = |x: f64| 0.4 + k * 0.0 + x;
            let p = |ang: f64, rr: f64| (ang.cos() * rr * 0.95, ang.sin() * rr);
            let (s0, s1, s2) = (p(a, rad(0.12)), p(b, rad(0.62)), p(e, rad(1.12)));
            let v = self.hp(&qc(pt(s0.0, s0.1), pt(s1.0, s1.1), pt(s2.0, s2.1), 7).iter().map(|q| (q.x, q.y)).collect::<Vec<_>>());
            v
        };
        let mut dk = vec![];
        let mut lt = vec![];
        for i in 0..n {
            let t = (i as f64 + 0.5) / n as f64;
            let jit = (hash(i as f64 * 2.3 + gnum(hair, "seed", 0.0)) - 0.5) * 6.0;
            let s = strand(i as f64, t, jit);
            lt.push(s.iter().map(|p| add(*p, light(), 0.35)).collect::<Vec<P>>());
            dk.push(s);
        }
        stroke_lines(ctx, &dk, shade_of(hcol), 0.5, 0.42);
        stroke_hi(ctx, &lt, light_of(hcol), 0.55, 0.32);
        let lk: Vec<Vec<P>> = (0..locks).map(|i| { let t = (i as f64 + 0.5) / locks as f64; strand(0.0, t, 4.0) }).collect();
        stroke_lines(ctx, &lk, self.c.c("outline"), 0.6, 0.5);
        if shine {
            let band: Vec<(f64, f64)> = (0..=10).map(|i| { let a = (205.0 + i as f64 * 9.0) * D2R; (a.cos() * 0.8, a.sin() * 0.8) }).chain((0..=10).rev().map(|i| { let a = (205.0 + i as f64 * 9.0) * D2R; (a.cos() * 0.66, a.sin() * 0.66 + 0.04) })).collect();
            fill_poly(ctx, &self.hp(&band), light_of(hcol), 0.26, true);
            let thin: Vec<(f64, f64)> = (0..=8).map(|i| { let a = (215.0 + i as f64 * 8.0) * D2R; (a.cos() * 0.74, a.sin() * 0.74) }).collect();
            stroke_lines(ctx, &[self.hp(&thin)], WHITE, 0.45, 0.3);
        }
        ctx.restore();
        let _ = r;
    }

    /// strands across a ribbon-like back hair mass (rails = the two long edges)
    pub(super) fn rail_strands(&self, ctx: &mut Ctx, a: &[P], b: &[P], col: Rgb, n: usize, clip: &Geo) {
        let m = a.len().min(b.len());
        if m < 3 || n == 0 { return; }
        ctx.save();
        clip_geo(ctx, std::slice::from_ref(clip));
        let mut dk = vec![];
        let mut lt = vec![];
        for k in 0..n {
            let f = (k as f64 + 0.5 + (hash(k as f64 * 3.7) - 0.5) * 0.5) / n as f64;
            let pl: Vec<P> = (1..m).map(|i| lerp(a[i], b[i], f)).collect();
            lt.push(pl.iter().map(|p| add(*p, light(), 0.4)).collect::<Vec<P>>());
            dk.push(pl);
        }
        stroke_lines(ctx, &dk, shade_of(col), 0.55, 0.4);
        stroke_hi(ctx, &lt, light_of(col), 0.5, 0.3);
        ctx.restore();
    }

    // ------------------------------------------------------------------ head gear shapes (outlined)
    pub(super) fn head_gear(&self) -> Vec<Shape> {
        let (c, style, sk) = (&self.c, self.style, &self.sk);
        let (r, s) = (sk.r, sk.s);
        let mut out = vec![];
        if gstr(style, "ears") == Some("elf") && !self.helmet {
            let skin = c.c("skin");
            let e = self.hp(&[(-0.02, 0.02), (-0.5, -0.1), (-1.0, -0.36), (-0.62, 0.12), (-0.1, 0.46)]);
            out.push(shp(poly(e, false), skin, 1.0).line(true));
            let inner = self.hp(&[(-0.1, 0.08), (-0.5, -0.02), (-0.8, -0.24), (-0.5, 0.14)]);
            let mut sh = shp(poly(inner, false), shade_of(skin), 0.0);
            sh.no_outline = true;
            sh.flat = true;
            out.push(sh);
        }
        if let Some(cr) = self.sv("crown") {
            let metal = self.col_of(gv(cr, "color"), c.c("accent"));
            let gem = self.col_of(gv(cr, "gem"), c.c("energy"));
            let n = gnum(cr, "spikes", 3.0).max(1.0) as i32;
            let band = self.hp(&[(-0.62, -0.7), (-0.25, -0.88), (0.3, -0.92), (0.75, -0.78), (0.9, -0.6), (0.82, -0.52), (0.3, -0.7), (-0.25, -0.66), (-0.58, -0.54)]);
            out.push(thin(shp(poly(band, true), metal, 1.0)));
            for i in 0..n {
                let t = (i as f64 + 0.5) / n as f64;
                let x = -0.4 + t * 1.2;
                let hgt = 0.46 + 0.5 * (1.0 - (t - 0.5).abs() * 2.0);
                let by = -0.9 + (x - 0.2).abs() * 0.2;
                let sp = self.hp(&[(x - 0.13, by + 0.08), (x, by - hgt), (x + 0.13, by + 0.08)]);
                out.push(thin(shp(poly(sp, false), light_of(metal), 0.8)));
                if i == n / 2 { let g = self.h(x * r, (by - hgt * 0.35) * r); out.push(thin(shp(Geo::Ellipse(g, 0.1 * r, 0.14 * r, 0.0), gem, 0.0))); }
            }
            let g = self.h(0.28 * r, -0.8 * r);
            out.push(thin(shp(Geo::Ellipse(g, 0.11 * r, 0.1 * r, 0.0), gem, 0.0)));
        }
        if let Some(h) = self.sv("hairpin") {
            let col = self.col_of(gv(h, "color"), c.c("accent"));
            let col2 = self.col_of(gv(h, "color2"), c.c("top2"));
            match gstr(h, "type").unwrap_or("sticks") {
                "flower" => {
                    let ctr = self.h(0.0, -0.98 * r);
                    for k in 0..5 {
                        let a = k as f64 / 5.0 * PI * 2.0;
                        out.push(thin(shp(Geo::Ellipse(add(ctr, pt(a.cos(), a.sin()), 0.38 * r * s), 0.24 * r, 0.14 * r, a), col, 0.6)));
                    }
                    out.push(thin(shp(Geo::Ellipse(ctr, 0.13 * r, 0.13 * r, 0.0), col2, 0.0)));
                }
                "ribbon" => {
                    let p = self.h(-0.72 * r, -0.72 * r);
                    out.push(shp(poly(vec![p, add(p, pt(-4.4 * s, -3.0 * s), 1.0), add(p, pt(-4.8 * s, 2.2 * s), 1.0)], false), col, 0.8).line(true));
                    out.push(shp(poly(vec![p, add(p, pt(4.4 * s, -3.4 * s), 1.0), add(p, pt(4.0 * s, 2.0 * s), 1.0)], false), col, 0.8).line(true));
                    out.push(shp(Geo::Ellipse(p, 1.3 * s, 1.3 * s, 0.0), col2, 0.4).line(true));
                }
                "ring" => {
                    let p = self.h(-0.78 * r, -0.62 * r);
                    out.push(thin(shp(Geo::Ellipse(p, 2.0 * s, 2.7 * s, sk.head_th), col, 1.0)));
                }
                _ => {
                    let k = self.h(-0.55 * r, -1.05 * r);
                    for (i, sgn) in [-1.0, 1.0].iter().enumerate() {
                        let a = add(k, pt(-6.2 * s, sgn * 3.0 * s), 1.0);
                        let b = add(k, pt(6.2 * s, sgn * -2.4 * s), 1.0);
                                        out.push(thin(shp(Geo::Capsule(a, b, 0.32 * s, 0.32 * s), if i == 0 { col } else { light_of(col) }, 0.4)));
                        out.push(thin(shp(Geo::Ellipse(b, 0.8 * s, 0.8 * s, 0.0), col2, 0.4)));
                    }
                }
            }
        }
        if let Some(g) = self.sv("goggles") {
            let fc = self.col_of(gv(g, "color"), [0x30, 0x2c, 0x38]);
            let lens = self.col_of(gv(g, "lens"), c.c("energy"));
            let strap = self.hp(&[(-0.98, -0.52), (-0.3, -0.7), (0.5, -0.68), (0.97, -0.5), (1.0, -0.36), (0.5, -0.5), (-0.3, -0.52), (-0.98, -0.34)]);
            out.push(shp(poly(strap, true), fc, 1.0).line(true));
            for (x, rr) in [(0.12, 0.3), (0.62, 0.26)] {
                let p = self.h(x * r, -0.6 * r);
                out.push(shp(Geo::Ellipse(p, rr * r * 1.05, rr * r, 0.0), fc, 1.0).line(true));
                let mut lz = shp(Geo::Ellipse(p, rr * r * 0.78, rr * r * 0.72, 0.0), lens, 0.0);
                lz.flat = true;
                out.push(lz);
                let mut gl = shp(Geo::Ellipse(add(p, pt(-0.1 * r, -0.12 * r), 1.0), rr * r * 0.25, rr * r * 0.14, -0.6), WHITE, 0.0);
                gl.no_outline = true;
                gl.flat = true;
                out.push(gl);
            }
        }
        out
    }
}
