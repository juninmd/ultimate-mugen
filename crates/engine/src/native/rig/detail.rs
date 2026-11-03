//! Optional physical detail for the vector rig: muscle definition, veins, cloth folds, stitching,
//! fabric weave, tattoos, bandages, gloves, boots. Every feature is opt-in through `art.style`
//! keys, so characters that do not ask for them render exactly as before. All geometry is a pure
//! function of the pose / style (deterministic, hash based), drawn after the base shapes of a layer.
//!
//! Style keys (all optional; colours are palette keys or `#hex`):
//! - proportions (visual only, hit/hurt boxes and sprite size are untouched): `arms`, `legs`, `chest`, `waist` (multipliers)
//! - body: `muscle` (0..1.6), `veins` (bool | colour), `tattoo {type: tribal|bands|flame|swirl|stars, on: [arm, forearm, chest, back, ribs], color}`,
//!   `scars [chest, ribs, brow, cheek, jaw, nose]`, `bandages [forearm, arm, calf, thigh, chest]`, `knuckles`
//! - cloth: `folds` (0..1.6), `foldSeed`, `stitch` (bool | colour), `weave` / `pantsWeave` (hatch|weave|check|rib|stripes|scales|chain|quilt|dots),
//!   `hem`, `emblem {type, color, where: chest|back, size}`, `sash {color, rounds, buckle}`, `necklace {type: beads|tag|chain|pendant}`,
//!   `pouches`, `talisman`, `buckle` (round|square|diamond|star|skull)
//! - armor: `armor {rivets, engrave: scroll|chevron|circuit|spikes}`, `pauldron` (round|layered|spiked|fur|horned), `helmetDetail`
//! - feet: `footwear {sole, laces, toeCap, cuff, buckles, split, wrapped}`
//! - face: `faceShade`, `brow {thick, shape, color}`, `beard` (stubble|goatee|full|moustache|chinstrap|sideburns), `glasses {type}`, `eyepatch`,
//!   `earring`, `facePaint {type}`, `ears: elf`, `crown`, `hairpin`, `goggles`; hair: `strands`, `locks`, `shine`
use super::*;

// ------------------------------------------------------------------ drawing helpers
/// polyline set -> single path (one rasterizer pass per batch)
pub(super) fn path_lines(ctx: &mut Ctx, ls: &[Vec<P>]) {
    ctx.begin();
    for l in ls { if l.len() < 2 { continue; } ctx.mv(l[0]); for p in &l[1..] { ctx.ln(*p); } }
}
/// secondary highlight strokes: only worth their cost on high resolution (4K) art
pub(super) fn stroke_hi(ctx: &mut Ctx, ls: &[Vec<P>], col: Rgb, a: f64, w: f64) { if res() >= 4.0 { stroke_lines(ctx, ls, col, a, w); } }
pub(super) fn stroke_lines(ctx: &mut Ctx, ls: &[Vec<P>], col: Rgb, a: f64, w: f64) {
    if ls.is_empty() || a <= 0.0 { return; }
    if w <= 0.8 { ctx.stroke_quads(ls, w, col, a); return; }
    path_lines(ctx, ls);
    ctx.st.butt = true;
    ctx.stroke_w(&sol(col, a), w);
    ctx.st.butt = false;
}
/// sampled quadratic bezier
pub(super) fn qc(a: P, c: P, b: P, n: usize) -> Vec<P> {
    (0..=n).map(|i| { let t = i as f64 / n as f64; let u = 1.0 - t; pt(u * u * a.x + 2.0 * u * t * c.x + t * t * b.x, u * u * a.y + 2.0 * u * t * c.y + t * t * b.y) }).collect()
}
/// dash a polyline into segments
pub(super) fn dashed(pts: &[P], dash: f64, gap: f64) -> Vec<Vec<P>> {
    let mut out = vec![];
    let (mut on, mut left) = (true, dash);
    let mut cur: Vec<P> = vec![];
    for w in pts.windows(2) {
        let (a, b) = (w[0], w[1]);
        let mut l = len(sub(b, a));
        if l <= 0.0 { continue; }
        let d = norm(sub(b, a));
        let mut p = a;
        if on && cur.is_empty() { cur.push(p); }
        while l > left {
            p = add(p, d, left);
            l -= left;
            if on { cur.push(p); out.push(std::mem::take(&mut cur)); left = gap; } else { cur.push(p); left = dash; }
            on = !on;
        }
        left -= l;
        if on { cur.push(b); }
    }
    if cur.len() > 1 { out.push(cur); }
    out
}
pub(super) fn fill_poly(ctx: &mut Ctx, pts: &[P], col: Rgb, a: f64, smooth: bool) {
    if pts.len() < 3 || a <= 0.0 { return; }
    ctx.begin();
    poly(pts.to_vec(), smooth).emit(ctx, 0.0, 0.0);
    ctx.fill(&sol(col, a));
}
pub(super) fn stroke_poly(ctx: &mut Ctx, pts: &[P], col: Rgb, a: f64, w: f64, smooth: bool) {
    if pts.len() < 3 || a <= 0.0 { return; }
    ctx.begin();
    poly(pts.to_vec(), smooth).emit(ctx, 0.0, 0.0);
    ctx.stroke_w(&sol(col, a), w);
}
pub(super) fn fill_ell(ctx: &mut Ctx, c: P, rx: f64, ry: f64, rot: f64, col: Rgb, a: f64) {
    if a <= 0.0 { return; }
    ctx.begin();
    Geo::Ellipse(c, rx, ry, rot).emit(ctx, 0.0, 0.0);
    ctx.fill(&sol(col, a));
}
/// many small discs in one path
pub(super) fn dots(ctx: &mut Ctx, ps: &[P], r: f64, col: Rgb, a: f64) {
    if ps.is_empty() || a <= 0.0 { return; }
    ctx.begin();
    for p in ps { ctx.move_to(p.x + r, p.y); ctx.arc(p.x, p.y, r, 0.0, PI * 2.0); ctx.close(); }
    ctx.fill(&sol(col, a));
}
pub(super) fn clip_geo(ctx: &mut Ctx, geos: &[Geo]) {
    ctx.begin();
    for g in geos { g.emit(ctx, 0.0, 0.0); }
    ctx.clip();
}
pub(super) fn ang(p: P) -> f64 { p.y.atan2(p.x) }
/// 5-point star outline
pub(super) fn star_pts(c: P, r: f64, rot: f64) -> Vec<P> {
    (0..10).map(|i| { let a = rot + i as f64 * PI / 5.0 - PI / 2.0; let rr = if i % 2 == 0 { r } else { r * 0.42 }; pt(c.x + a.cos() * rr, c.y + a.sin() * rr) }).collect()
}

/// a limb segment with its lit-side normal
#[derive(Clone, Copy)]
pub(super) struct Limb { pub a: P, pub b: P, pub u: P, pub n: P, pub l: f64 }
impl Limb {
    pub fn new(a: P, b: P) -> Limb {
        let u = norm(sub(b, a));
        let mut n = perp(u);
        let lt = light();
        if n.x * lt.x + n.y * lt.y < 0.0 { n = pt(-n.x, -n.y); }
        Limb { a, b, u, n, l: len(sub(b, a)) }
    }
    /// point at fraction t along the limb, `off` towards the lit side (negative = shadow side)
    pub fn at(&self, t: f64, off: f64) -> P { add(lerp(self.a, self.b, t), self.n, off) }
    pub fn rot(&self) -> f64 { ang(self.u) }
}

fn vein_col(skin: Rgb) -> Rgb { mix(skin, [0x4a, 0x5a, 0xa8], 0.42) }

impl<'a> Rig<'a> {
    pub(super) fn sn(&self, k: &str, d: f64) -> f64 { gnum(self.style, k, d) }
    pub(super) fn ss(&self, k: &str) -> Option<&str> { gstr(self.style, k) }
    pub(super) fn sv(&self, k: &str) -> Option<&Value> { gv(self.style, k).filter(|v| truthy(Some(v))) }
    /// colour from a style value (palette key or hex); `true` / missing -> `def`
    pub(super) fn col_of(&self, v: Option<&Value>, def: Rgb) -> Rgb {
        match v.and_then(|x| x.as_str()) { Some(s) => self.c.get(s).unwrap_or_else(|| parse_rgb(s)), None => def }
    }
    pub(super) fn stitch_col(&self, base: Rgb) -> Option<Rgb> {
        let v = self.sv("stitch")?;
        Some(self.col_of(Some(v), mix(base, if lum(base) > 0.45 { [0x30, 0x20, 0x20] } else { [0xff, 0xf0, 0xd0] }, 0.55)))
    }
    pub(super) fn arm_radii(&self) -> (f64, f64, f64, f64) {
        let (w, k) = (self.sk.w, self.sn("arms", 1.0));
        let fm = |a: f64, b: f64| if self.sk.fem { a } else { b };
        (3.8 * w * fm(0.82, 1.0) * k, 3.0 * w * fm(0.82, 1.0) * k, 3.2 * w * fm(0.82, 1.0) * k, 2.5 * w * fm(0.85, 1.0) * k)
    }
    pub(super) fn leg_scale(&self) -> f64 { self.sn("legs", 1.0) }
    pub(super) fn wide_sleeve(&self, front: bool) -> Vec<P> {
        let sk = &self.sk;
        let (sh, el, wr) = if front { (sk.sh_f, sk.el_f, sk.wr_f) } else { (sk.sh_b, sk.el_b, sk.wr_b) };
        let (w, s) = (sk.w, sk.s);
        let g = pt(0.0, 1.0);
        let d = norm(sub(wr, el));
        let n = norm(perp(sub(el, sh)));
        let s_end = add(el, d, sk.l.fore * 0.55);
        let n2 = norm(perp(d));
        let hang = add(g, dir(self.trail), 0.3);
        vec![add(sh, n, 4.5 * w), add(el, n, 5.0 * w), add(s_end, n2, 6.5 * w), add(add(s_end, n2, -6.5 * w), hang, 7.0 * s), add(add(el, n, -5.5 * w), hang, 4.0 * s), add(sh, n, -4.5 * w)]
    }

    // ------------------------------------------------------------------ fabric
    /// fabric texture over `[o, o + u*lu + v*lv]` (call under a clip): a tiny repeating tile (rendered once
    /// per kind / colour / art resolution) is mapped onto the area with a pattern shader, so the cost is
    /// proportional to the pixels covered instead of to the number of thread lines.
    pub(super) fn weave(&self, ctx: &mut Ctx, kind: &str, base: Rgb, area: &[Geo], o: P, u: P, al: f64) {
        let Some(tile) = fabric_tile(kind, base) else { return };
        ctx.begin();
        for g in area { g.emit(ctx, 0.0, 0.0); }
        ctx.fill_tile(&tile, o, ang(u), al);
    }

    /// stitched seam along a polyline (dashed, with a light companion line)
    pub(super) fn seam(&self, ctx: &mut Ctx, pts: &[P], col: Rgb, a: f64) {
        let d = dashed(pts, 1.1, 0.8);
        stroke_lines(ctx, &d, col, a, 0.24);
    }

    /// a wrap / bandage strip along a limb section (call under the limb clip)
    pub(super) fn wraps(&self, ctx: &mut Ctx, l: &Limb, t0: f64, t1: f64, r0: f64, r1: f64, col: Rgb, al: f64) {
        let rr = |t: f64| r0 + (r1 - r0) * t;
        let edge = |s: f64| -> Vec<P> { (0..=6).map(|i| { let t = t0 + (t1 - t0) * i as f64 / 6.0; l.at(t, s * rr(t) * 1.25) }).collect() };
        let mut poly_pts = edge(1.0);
        let mut other = edge(-1.0);
        other.reverse();
        poly_pts.extend(other);
        fill_poly(ctx, &poly_pts, col, al, false);
        let n = (((t1 - t0) * l.l) / 2.2).ceil().max(2.0) as usize;
        let mut dk = vec![];
        let mut hi = vec![];
        for i in 0..n {
            let t = t0 + (t1 - t0) * (i as f64 + 0.5) / n as f64;
            let r = rr(t) * 1.25;
            dk.push(vec![l.at(t - 0.04, -r), l.at(t + 0.05, r)]);
            hi.push(vec![add(l.at(t - 0.04, -r), l.u, 0.28), add(l.at(t + 0.05, r), l.u, 0.28)]);
        }
        stroke_lines(ctx, &dk, shade_of(col), 0.7 * al, 0.34);
        stroke_hi(ctx, &hi, light_of(col), 0.5 * al, 0.2);
        stroke_lines(ctx, &[edge(1.0), edge(-1.0)], shade_of(col), 0.5 * al, 0.25);
    }

    fn tattoo_on(&self, ctx: &mut Ctx, l: &Limb, t0: f64, t1: f64, r: f64, kind: &str, col: Rgb, al: f64) {
        match kind {
            "bands" => {
                for (i, t) in [t0 + (t1 - t0) * 0.25, t0 + (t1 - t0) * 0.42, t0 + (t1 - t0) * 0.5].iter().enumerate() {
                    let w = if i == 2 { 0.5 } else { 0.9 };
                    fill_poly(ctx, &[l.at(*t, -r * 1.2), l.at(*t, r * 1.2), l.at(*t + w / l.l.max(1.0), r * 1.2), l.at(*t + w / l.l.max(1.0), -r * 1.2)], col, al, false);
                }
            }
            "flame" => {
                for k in 0..3 {
                    let b = t0 + (t1 - t0) * (0.15 + k as f64 * 0.3);
                    let o = (k as f64 - 1.0) * r * 0.55;
                    let tip = l.at(b + 0.32 - k as f64 * 0.04, o + r * 0.35);
                    fill_poly(ctx, &[l.at(b, o - r * 0.5), l.at(b + 0.12, o - r * 0.2 + r * 0.15), tip, l.at(b + 0.14, o + r * 0.55), l.at(b, o + r * 0.5)], col, al, true);
                }
            }
            "swirl" => {
                let c = l.at((t0 + t1) * 0.5, 0.0);
                let ls: Vec<Vec<P>> = vec![(0..=22).map(|i| { let a = i as f64 * 0.42; let rr = r * (0.2 + i as f64 * 0.04); pt(c.x + a.cos() * rr, c.y + a.sin() * rr) }).collect()];
                stroke_lines(ctx, &ls, col, al, 0.5);
            }
            "stars" => {
                for k in 0..3 { let c = l.at(t0 + (t1 - t0) * (0.2 + k as f64 * 0.3), (k as f64 - 1.0) * r * 0.5); fill_poly(ctx, &star_pts(c, r * 0.4, k as f64), col, al, false); }
            }
            _ => {
                // tribal: zig-zag armband with barbs
                let n = 7;
                let mut a = vec![];
                let mut b = vec![];
                for i in 0..=n {
                    let t = t0 + (t1 - t0) * 0.3 + (t1 - t0) * 0.4 * i as f64 / n as f64;
                    let o = if i % 2 == 0 { r * 1.05 } else { r * 0.45 };
                    a.push(l.at(t, o));
                    b.push(l.at(t, -o));
                }
                let mut pts = a;
                b.reverse();
                pts.extend(b);
                fill_poly(ctx, &pts, col, al, false);
                let spine: Vec<P> = (0..=n).map(|i| l.at(t0 + (t1 - t0) * 0.3 + (t1 - t0) * 0.4 * i as f64 / n as f64, 0.0)).collect();
                stroke_lines(ctx, &[spine], mix(col, skin_tint(), 0.55), al * 0.9, 0.3);
                fill_poly(ctx, &[l.at(t0 + (t1 - t0) * 0.7, r * 0.25), l.at(t1 - 0.02, 0.0), l.at(t0 + (t1 - t0) * 0.7, -r * 0.25)], col, al, false);
            }
        }
    }

    // ------------------------------------------------------------------ arms
    pub(super) fn arm_detail(&self, ctx: &mut Ctx, front: bool) {
        let sk = &self.sk;
        let (sh, el, wr) = if front { (sk.sh_f, sk.el_f, sk.wr_f) } else { (sk.sh_b, sk.el_b, sk.wr_b) };
        let al = if front { 1.0 } else { 0.72 };
        let (ur0, ur1, fr0, fr1) = self.arm_radii();
        let skin = self.c.c("skin");
        let top_c = self.c.c("top");
        let sl = self.sleeves.as_str();
        // the far limb is mostly hidden behind the body: it only gets the cheap features
        let (m, folds) = (if front { self.sn("muscle", 0.0) * 1.3 } else { 0.0 }, self.sn("folds", 0.0));
        let weave = if front { self.ss("weave").unwrap_or("") } else { "" };
        let stitch = if front { self.stitch_col(top_c) } else { None };
        let up_skin = sl == "none";
        let fo_skin = matches!(sl, "none" | "short" | "gi");
        let rad = |a: f64, b: f64, t: f64| a + (b - a) * t;
        let up_piece = |t0: f64, t1: f64| Geo::Capsule(lerp(sh, el, t0), lerp(sh, el, t1), rad(ur0, ur1, t0), rad(ur0, ur1, t1));
        let fo_piece = |t0: f64, t1: f64| Geo::Capsule(lerp(el, wr, t0), lerp(el, wr, t1), rad(fr0, fr1, t0), rad(fr0, fr1, t1));
        let mut skin_geos = vec![];
        if up_skin { skin_geos.push(up_piece(0.0, 1.0)); }
        if fo_skin { skin_geos.push(fo_piece(if sl == "gi" { 0.38 } else { 0.0 }, 1.0)); }
        let mut cloth_geos = vec![];
        match sl { "long" => { cloth_geos.push(up_piece(0.0, 1.0)); cloth_geos.push(fo_piece(0.0, 1.0)); } "short" => cloth_geos.push(up_piece(0.0, 0.72)), "gi" => { cloth_geos.push(up_piece(0.0, 1.0)); cloth_geos.push(fo_piece(0.0, 0.36)); } _ => {} }
        let ul = Limb::new(sh, el);
        let fl = Limb::new(el, wr);
        let bandages = self.bandage_list();
        let tat = self.sv("tattoo");
        let cloth_up = matches!(sl, "long" | "gi" | "short");
        let skin_any = m > 0.0 || tat.is_some() || !bandages.is_empty() || (front && self.sv("veins").is_some());
        if skin_any && !skin_geos.is_empty() {
            ctx.save();
            clip_geo(ctx, &skin_geos);
            let vc = self.col_of(self.sv("veins").filter(|v| v.is_string()), vein_col(skin));
            let (hi, shd) = (mix(skin, [0xff, 0xf6, 0xe8], 0.5), mix(skin, [0x2b, 0x17, 0x48], 0.5));
            // ---- muscle
            if m > 0.0 {
                if up_skin {
                    fill_ell(ctx, ul.at(0.42, ur0 * 0.34), ul.l * 0.2, ur0 * 0.3, ul.rot(), hi, 0.34 * m.min(1.5) * al);
                    stroke_lines(ctx, &[qc(ul.at(0.14, -ur0 * 0.78), ul.at(0.5, -ur0 * 0.95), ul.at(0.92, -ur1 * 0.8), 6)], shd, 0.5 * m.min(1.3) * al, 0.6);
                    stroke_lines(ctx, &[qc(ul.at(0.2, ur0 * 0.02), ul.at(0.5, -ur0 * 0.12), ul.at(0.82, -ur1 * 0.05), 6)], shd, 0.3 * m.min(1.3) * al, 0.28);
                    // deltoid cap + tricep sweep
                    stroke_lines(ctx, &[qc(ul.at(0.1, ur0 * 1.1), ul.at(0.34, ur0 * 0.1), ul.at(0.12, -ur0 * 1.1), 6)], shd, 0.38 * m.min(1.3) * al, 0.34);
                    stroke_lines(ctx, &[qc(ul.at(0.4, -ur0 * 0.35), ul.at(0.62, -ur0 * 0.6), ul.at(0.9, -ur1 * 0.3), 6)], hi, 0.22 * m.min(1.3) * al, 0.3);
                }
                if fo_skin {
                    fill_ell(ctx, fl.at(0.28, fr0 * 0.3), fl.l * 0.2, fr0 * 0.34, fl.rot(), hi, 0.3 * m.min(1.5) * al);
                    let ts: Vec<Vec<P>> = (0..3).map(|i| { let o = (i as f64 - 1.0) * fr1 * 0.5; vec![fl.at(0.9, o * 0.7), fl.at(0.6, o), fl.at(0.46, o * 1.2)] }).collect();
                    stroke_lines(ctx, &ts, shd, 0.42 * m.min(1.3) * al, 0.24);
                    stroke_lines(ctx, &[qc(fl.at(0.1, -fr0 * 0.8), fl.at(0.4, -fr0 * 0.9), fl.at(0.85, -fr1 * 0.7), 6)], shd, 0.4 * m.min(1.3) * al, 0.5);
                }
            }
            // ---- veins
            if front && self.sv("veins").is_some() {
                let wav = |l: &Limb, t0: f64, t1: f64, off: f64, amp: f64, ph: f64| -> Vec<P> { (0..=9).map(|i| { let t = t0 + (t1 - t0) * i as f64 / 9.0; l.at(t, off + (t * 17.0 + ph).sin() * amp) }).collect() };
                let mut vs = vec![];
                if fo_skin { vs.push(wav(&fl, 0.12, 0.92, fr0 * 0.28, fr0 * 0.2, 0.0)); vs.push(wav(&fl, 0.5, 0.88, -fr0 * 0.2, fr0 * 0.15, 2.0)); }
                if up_skin { vs.push(wav(&ul, 0.55, 0.98, ur0 * 0.18, ur0 * 0.16, 1.0)); vs.push(wav(&ul, 0.3, 0.7, ur0 * 0.5, ur0 * 0.12, 3.5)); }
                stroke_lines(ctx, &vs, vc, 0.62 * al, 0.2);
                let br: Vec<Vec<P>> = vs.iter().filter_map(|v| if v.len() > 5 { Some(vec![v[4], add(add(v[4], ul.n, 0.9), ul.u, -0.9)]) } else { None }).collect();
                stroke_lines(ctx, &br, vc, 0.45 * al, 0.15);
            }
            // ---- tattoo / bandage
            if let Some(t) = tat {
                let tc = self.col_of(gv(t, "color"), mix(skin, [0x28, 0x2a, 0x60], 0.7));
                let kind = gstr(t, "type").unwrap_or("tribal");
                let sites = gv(t, "on").and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>()).unwrap_or_else(|| vec!["arm"]);
                if sites.contains(&"arm") && up_skin { self.tattoo_on(ctx, &ul, 0.18, 0.95, ur0 * 0.92, kind, tc, 0.85 * al); }
                if sites.contains(&"forearm") && fo_skin { self.tattoo_on(ctx, &fl, 0.1, 0.95, fr0 * 0.92, kind, tc, 0.85 * al); }
            }
            for b in &bandages {
                let wc = self.c.get("wrap").unwrap_or([0xec, 0xe6, 0xd8]);
                match b.as_str() {
                    "forearm" => self.wraps(ctx, &fl, 0.12, 0.62, fr0, fr1, wc, 0.95 * al),
                    "arm" | "upperArm" => self.wraps(ctx, &ul, 0.45, 0.85, ur0, ur1, wc, 0.95 * al),
                    _ => {}
                }
            }
            ctx.restore();
        }
        let cloth_any = folds > 0.0 || !weave.is_empty() || stitch.is_some();
        if cloth_any && !cloth_geos.is_empty() {
            ctx.save();
            clip_geo(ctx, &cloth_geos);
            if cloth_up {
                if !weave.is_empty() {
                    self.weave(ctx, weave, top_c, &cloth_geos, sh, ul.u, 0.9 * al);
                }
                if folds > 0.0 {
                    let f = folds.min(1.6);
                    let inner = norm(sub(lerp(sh, wr, 0.5), el));
                    for (k, t) in [0.8, 0.88, 0.95].iter().enumerate() {
                        let kk = k as f64;
                        let a = ul.at(*t, -ur1 * 1.0);
                        let b = ul.at(*t + 0.02, ur1 * 1.0);
                        let c = add(lerp(a, b, 0.5), inner, ur1 * (0.5 + kk * 0.2));
                        stroke_lines(ctx, &[qc(a, c, b, 6)], shade_of(top_c), 0.55 * f.min(1.0) * al, 0.38);
                        stroke_hi(ctx, &[qc(add(a, ul.u, 0.45), add(c, ul.u, 0.45), add(b, ul.u, 0.45), 6)], light_of(top_c), 0.26 * f.min(1.0) * al, 0.25);
                    }
                    for (k, t) in [0.05, 0.14, 0.22].iter().enumerate() {
                        let a = fl.at(*t, -fr0 * 1.0);
                        let b = fl.at(*t - 0.01, fr0 * 1.0);
                        let c = add(lerp(a, b, 0.5), inner, fr0 * (0.35 + k as f64 * 0.15));
                        stroke_lines(ctx, &[qc(a, c, b, 6)], shade_of(top_c), 0.45 * f.min(1.0) * al, 0.34);
                    }
                    // cuff bunching
                    let cuffs: Vec<Vec<P>> = (0..3).map(|k| { let t = 0.8 + k as f64 * 0.07; vec![fl.at(t, -fr1 * 1.1), add(fl.at(t + 0.01, 0.0), fl.u, -0.35), fl.at(t, fr1 * 1.1)] }).collect();
                    stroke_lines(ctx, &cuffs, shade_of(top_c), 0.5 * f.min(1.0) * al, 0.3);
                    // shoulder drag lines
                    let sd: Vec<Vec<P>> = (0..3).map(|k| { let o = (k as f64 - 1.0) * ur0 * 0.55; vec![ul.at(0.05, o), ul.at(0.3, o * 0.7 - ur0 * 0.1)] }).collect();
                    stroke_lines(ctx, &sd, shade_of(top_c), 0.32 * f.min(1.0) * al, 0.28);
                    fill_poly(ctx, &[el, add(el, inner, ur1 * 1.4), add(ul.at(0.82, 0.0), inner, ur1 * 0.2)], shade_of(top_c), 0.2 * f.min(1.0) * al, true);
                }
                if let Some(sc) = stitch {
                    let seam: Vec<P> = (0..=8).map(|i| ul.at(0.08 + 0.9 * i as f64 / 8.0, -ur1 * 0.72)).collect();
                    self.seam(ctx, &seam, sc, 0.8 * al);
                    let seam2: Vec<P> = (0..=6).map(|i| fl.at(0.05 + 0.85 * i as f64 / 6.0, -fr1 * 0.7)).collect();
                    self.seam(ctx, &seam2, sc, 0.8 * al);
                    let ring: Vec<P> = (0..=4).map(|i| fl.at(0.86, -fr1 * 1.1 + fr1 * 2.2 * i as f64 / 4.0)).collect();
                    self.seam(ctx, &ring, sc, 0.8 * al);
                }
            }
            ctx.restore();
        }
        // wide sleeve folds on its own polygon
        if sl == "wide" && (folds > 0.0 || !weave.is_empty() || stitch.is_some()) {
            let sp = self.wide_sleeve(front);
            ctx.save();
            clip_geo(ctx, &[Geo::Poly(sp.clone(), true)]);
            if !weave.is_empty() { self.weave(ctx, weave, top_c, &[Geo::Poly(sp.clone(), true)], sh, ul.u, 0.9 * al); }
            if folds > 0.0 {
                let f = folds.min(1.6).min(1.0);
                // drape lines from the shoulder/elbow hem down to the hanging cuff
                let hang = [sp[1], sp[2], sp[3], sp[4]];
                let ls: Vec<Vec<P>> = (0..5).map(|k| {
                    let t = (k as f64 + 0.6) / 5.5;
                    let a = lerp(sp[0], sp[5], 1.0 - t);
                    let top = lerp(sp[1], sp[4], t);
                    let tip = lerp(hang[1], hang[2], t);
                    vec![a, top, add(lerp(top, tip, 0.55), pt(0.0, 1.0), 0.6 + hash(k as f64 + 4.0) * 0.8), tip]
                }).collect();
                stroke_lines(ctx, &ls.iter().map(|l| qc(l[1], l[2], l[3], 8)).collect::<Vec<_>>(), shade_of(top_c), 0.5 * f * al, 0.45);
                stroke_hi(ctx, &ls.iter().map(|l| qc(add(l[1], light(), 0.6), add(l[2], light(), 0.6), add(l[3], light(), 0.6), 8)).collect::<Vec<_>>(), light_of(top_c), 0.22 * f * al, 0.3);
            }
            if let Some(sc) = stitch { self.seam(ctx, &[lerp(sp[3], sp[2], 0.04), lerp(sp[3], sp[2], 0.96)].iter().map(|p| add(*p, pt(0.0, -1.0), 1.0)).collect::<Vec<_>>(), sc, 0.8 * al); }
            ctx.restore();
        }
        self.hand_detail(ctx, front);
    }

    pub(super) fn bandage_list(&self) -> Vec<String> {
        gv(self.style, "bandages").and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect()).unwrap_or_default()
    }

    /// knuckles, finger creases, gauntlet plates
    fn hand_detail(&self, ctx: &mut Ctx, front: bool) {
        let sk = &self.sk;
        let (el, wr, hd, fa) = if front { (sk.el_f, sk.wr_f, sk.hand_f, sk.fore_af) } else { (sk.el_b, sk.wr_b, sk.hand_b, sk.fore_ab) };
        let al = if front { 1.0 } else { 0.72 };
        let glove = gstr(self.style, "gloves").unwrap_or("none");
        let kn = self.sv("knuckles").is_some();
        let arm = self.sv("armor");
        let (_, _, fr0, fr1) = self.arm_radii();
        let skin = self.c.c("skin");
        let hc = if glove == "none" || glove == "wrap" { skin } else { self.c.c("glove") };
        let open = gstr(self.pose, if front { "handF" } else { "handB" }) == Some("open");
        let hr = sk.l.hand;
        let d = dir(fa + gnum(self.pose, if front { "wristF" } else { "wristB" }, 0.0));
        let n = perp(d);
        let nl = if n.x * light().x + n.y * light().y > 0.0 { n } else { pt(-n.x, -n.y) };
        if kn && !open {
            let c = add(hd, d, hr * 0.5);
            // knuckle bumps on the lit edge + dark creases between the fingers
            let ps: Vec<P> = (0..4).map(|i| add(add(c, nl, hr * 0.16), n, (i as f64 - 1.5) * hr * 0.4 * if (n.x * nl.x + n.y * nl.y) > 0.0 { 1.0 } else { -1.0 })).collect();
            dots(ctx, &ps, hr * 0.17, light_of(hc), 0.5 * al);
            let cr: Vec<Vec<P>> = (0..3).map(|i| { let p = add(add(c, nl, hr * 0.1), n, (i as f64 - 1.0) * hr * 0.4 + hr * 0.2); vec![add(p, d, -hr * 0.55), add(p, d, hr * 0.5)] }).collect();
            stroke_lines(ctx, &cr, shade_of(hc), 0.65 * al, 0.18);
            // thumb line
            stroke_lines(ctx, &[qc(add(hd, d, -hr * 0.7), add(add(hd, d, -hr * 0.1), nl, -hr * 0.6), add(hd, d, hr * 0.35), 6)], shade_of(hc), 0.55 * al, 0.2);
            if glove == "glove" || glove == "gauntlet" || glove == "long" {
                let seam: Vec<P> = (0..=4).map(|i| add(add(hd, d, -hr * 0.6 + hr * 0.3 * i as f64), nl, hr * 0.55)).collect();
                stroke_lines(ctx, &[seam], shade_of(hc), 0.5 * al, 0.15);
            }
        }
        if glove == "wrap" && kn {
            let c = lerp(wr, hd, 0.5);
            let ls: Vec<Vec<P>> = (0..4).map(|i| { let t = (i as f64 - 1.5) * hr * 0.5; vec![add(add(c, d, t), n, -hr * 1.0), add(add(c, d, t + hr * 0.35), n, hr * 1.0)] }).collect();
            stroke_lines(ctx, &ls, shade_of(self.c.get("wrap").unwrap_or([0xec, 0xe6, 0xd8])), 0.6 * al, 0.22);
        }
        if let (true, Some(a)) = (glove == "gauntlet", arm) {
            // plated vambrace: segment ridges, rivets and a lit edge
            let fl = Limb::new(el, wr);
            let (t0, r0, r1) = (0.4, fr0 * 1.18, fr1 * 1.22);
            let rr = |t: f64| r0 + (r1 - r0) * ((t - t0) / (1.0 - t0));
            let m = self.c.c("glove");
            ctx.save();
            clip_geo(ctx, &[Geo::Capsule(lerp(el, wr, t0), wr, r0, r1)]);
            let edge_l: Vec<P> = (0..=6).map(|i| { let t = t0 + (1.0 - t0) * i as f64 / 6.0; fl.at(t, rr(t) * 0.78) }).collect();
            stroke_lines(ctx, &[edge_l], light_of(m), 0.7 * al, 0.35);
            let rid: Vec<Vec<P>> = [0.58, 0.8].iter().map(|t| vec![fl.at(*t, -rr(*t) * 1.2), fl.at(*t + 0.01, 0.0), fl.at(*t, rr(*t) * 1.2)]).collect();
            stroke_lines(ctx, &rid, shade_of(m), 0.8 * al, 0.42);
            let rid_h: Vec<Vec<P>> = rid.iter().map(|l| l.iter().map(|p| add(*p, fl.u, 0.4)).collect()).collect();
            stroke_hi(ctx, &rid_h, light_of(m), 0.45 * al, 0.22);
            if gt(a, "rivets") {
                let rv: Vec<P> = [0.5, 0.7, 0.9].iter().map(|t| fl.at(*t, rr(*t) * 0.1)).collect();
                dots(ctx, &rv, 0.42, shade_of(m), 0.9 * al);
                let rh: Vec<P> = rv.iter().map(|p| add(*p, light(), 0.12)).collect();
                dots(ctx, &rh, 0.22, light_of(m), 0.8 * al);
            }
            ctx.restore();
        }
    }

    // ------------------------------------------------------------------ legs
    pub(super) fn leg_detail(&self, ctx: &mut Ctx, front: bool) {
        let sk = &self.sk;
        let (hp, kn, an) = if front { (sk.hip_f, sk.kn_f, sk.an_f) } else { (sk.hip_b, sk.kn_b, sk.an_b) };
        let al = if front { 1.0 } else { 0.72 };
        let w = sk.w;
        let lk = self.leg_scale();
        let skin = self.c.c("skin");
        let pants_c = self.c.c("pants");
        let pt_ = self.pants.as_str();
        let leg_skin = matches!(pt_, "trunks" | "skirt" | "shorts");
        let leg_col = if leg_skin { self.c.get("legs").unwrap_or(skin) } else { pants_c };
        let baggy = match pt_ { "gi" => 1.28, "hakama" => 1.5, "baggy" => 1.35, _ => 1.0 };
        let bg = if leg_skin { 1.0 } else { baggy };
        let (th_r0, th_r1, sh_r0, sh_r1) = (6.0 * w * if sk.fem { 0.92 } else { 1.0 } * lk, 4.3 * w * lk, 4.4 * w * lk, 2.9 * w * lk);
        let (m, folds) = (if front { self.sn("muscle", 0.0) * 1.3 } else { 0.0 }, self.sn("folds", 0.0));
        let weave = if front { self.ss("pantsWeave").unwrap_or("") } else { "" };
        let stitch = if front { self.stitch_col(leg_col) } else { None };
        let th = Limb::new(hp, kn);
        let sh = Limb::new(kn, an);
        let bandages = self.bandage_list();
        let any = m > 0.0 || folds > 0.0 || !weave.is_empty() || stitch.is_some() || !bandages.is_empty() || (front && self.sv("veins").is_some());
        if any {
            ctx.save();
            // boots cover the lower leg (pants that hang over them keep their detail)
            let skin_end = if baggy > 1.0 { 1.0 } else { match gstr(self.style, "boots").unwrap_or("boot") { "tall" => 0.06, "greave" => 0.26, "boot" => 0.46, "wrap" | "tabi" => 0.72, _ => 1.0 } };
            let shin_end = lerp(kn, an, skin_end);
            let tall = gstr(self.style, "boots") == Some("tall");
            let th1 = if tall { 0.76 } else { 1.0 };
            let th0 = if leg_skin { match pt_ { "trunks" => 0.22, "shorts" => 0.52, _ => 0.0 } } else { 0.0 };
            let leg_geos = [Geo::Capsule(lerp(hp, kn, th0), lerp(hp, kn, th1), (th_r0 + (th_r1 - th_r0) * th0) * bg, (th_r0 + (th_r1 - th_r0) * th1) * bg), Geo::Capsule(kn, shin_end, sh_r0 * bg, (sh_r0 + (sh_r1 - sh_r0) * skin_end) * bg)];
            clip_geo(ctx, &leg_geos);
            let (hi, shd) = (light_of(leg_col), shade_of(leg_col));
            let mm = if leg_skin { m } else if baggy > 1.0 { 0.0 } else { m * 0.55 };
            if mm > 0.0 {
                let r0 = th_r0 * bg;
                // quad sweep, vastus split and hamstring shade
                fill_ell(ctx, th.at(0.4, r0 * 0.3), th.l * 0.3, r0 * 0.3, th.rot(), hi, 0.3 * mm.min(1.5) * al);
                stroke_lines(ctx, &[qc(th.at(0.1, r0 * 0.1), th.at(0.45, -r0 * 0.05), th.at(0.92, -th_r1 * bg * 0.2), 7)], shd, 0.42 * mm.min(1.3) * al, 0.3);
                stroke_lines(ctx, &[qc(th.at(0.12, -r0 * 0.8), th.at(0.5, -r0 * 0.95), th.at(0.95, -th_r1 * bg * 0.8), 7)], shd, 0.5 * mm.min(1.3) * al, 0.7);
                // kneecap
                stroke_lines(ctx, &[qc(th.at(0.93, th_r1 * bg * 0.7), th.at(1.02, th_r1 * bg * 0.1), th.at(0.92, -th_r1 * bg * 0.5), 6)], shd, 0.4 * mm.min(1.3) * al, 0.3);
                // calf bulge + shin ridge
                let r1 = sh_r0 * bg;
                fill_ell(ctx, sh.at(0.3, -r1 * 0.35), sh.l * 0.24, r1 * 0.3, sh.rot(), hi, 0.3 * mm.min(1.5) * al);
                stroke_lines(ctx, &[qc(sh.at(0.12, r1 * 0.6), sh.at(0.5, r1 * 0.8), sh.at(0.9, sh_r1 * bg * 0.6), 7)], shd, 0.4 * mm.min(1.3) * al, 0.45);
                stroke_lines(ctx, &[qc(sh.at(0.2, -r1 * 0.1), sh.at(0.5, -r1 * 0.15), sh.at(0.78, -sh_r1 * bg * 0.1), 6)], hi, 0.28 * mm.min(1.3) * al, 0.3);
            }
            if front && self.sv("veins").is_some() && leg_skin {
                let vc = vein_col(skin);
                let v: Vec<P> = (0..=9).map(|i| { let t = 0.2 + 0.7 * i as f64 / 9.0; sh.at(t, sh_r0 * 0.3 + (t * 15.0).sin() * sh_r0 * 0.15) }).collect();
                stroke_lines(ctx, &[v], vc, 0.5 * al, 0.2);
            }
            if !weave.is_empty() {
                self.weave(ctx, weave, pants_c, &leg_geos, hp, th.u, 0.9 * al);
            }
            if folds > 0.0 && !leg_skin {
                let f = folds.min(1.0);
                let kneeb = norm(sub(lerp(hp, an, 0.5), kn));
                // knee crease chevrons + thigh pull lines + ankle bunching
                for k in 0..3 {
                    let kk = k as f64;
                    let a = th.at(0.84 + kk * 0.06, -th_r1 * bg);
                    let b = th.at(0.84 + kk * 0.06, th_r1 * bg);
                    let c = add(lerp(a, b, 0.5), kneeb, th_r1 * bg * (0.55 + kk * 0.25));
                    stroke_lines(ctx, &[qc(a, c, b, 6)], shade_of(pants_c), 0.55 * f * al, 0.38);
                    stroke_hi(ctx, &[qc(add(a, th.u, 0.5), add(c, th.u, 0.5), add(b, th.u, 0.5), 6)], light_of(pants_c), 0.24 * f * al, 0.26);
                }
                let tl: Vec<Vec<P>> = (0..4).map(|k| { let o = (k as f64 - 1.5) * th_r0 * bg * 0.5; vec![th.at(0.05, o), th.at(0.28, o * 0.6 - th_r0 * 0.12)] }).collect();
                stroke_lines(ctx, &tl, shade_of(pants_c), 0.4 * f * al, 0.3);
                let bunch: Vec<Vec<P>> = (0..3).map(|k| { let t = 0.72 + k as f64 * 0.06; vec![sh.at(t, -sh_r1 * bg), add(sh.at(t + 0.015, 0.0), sh.u, -0.3), sh.at(t, sh_r1 * bg)] }).collect();
                stroke_lines(ctx, &bunch, shade_of(pants_c), 0.45 * f * al, 0.3);
                fill_poly(ctx, &[kn, add(kn, kneeb, th_r1 * bg * 1.4), th.at(0.82, th_r1 * bg * 0.2)], shade_of(pants_c), 0.2 * f * al, true);
                // inner thigh shadow
                stroke_lines(ctx, &[qc(th.at(0.05, -th_r0 * bg * 0.75), th.at(0.5, -th_r0 * bg * 0.9), th.at(0.9, -th_r1 * bg * 0.75), 6)], shade_of(pants_c), 0.3 * f * al, 0.9);
            }
            if let Some(sc) = stitch {
                if !leg_skin {
                    let s1: Vec<P> = (0..=9).map(|i| th.at(0.06 + 0.88 * i as f64 / 9.0, th_r0 * bg * 0.05 + (th_r1 - th_r0) * bg * 0.05 * i as f64 / 9.0 - th_r0 * bg * 0.62)).collect();
                    self.seam(ctx, &s1, sc, 0.75 * al);
                    let s2: Vec<P> = (0..=8).map(|i| sh.at(0.08 + 0.84 * i as f64 / 8.0, -sh_r0 * bg * 0.6)).collect();
                    self.seam(ctx, &s2, sc, 0.75 * al);
                    // knee patch
                    let kp = [th.at(0.84, -th_r1 * bg * 0.85), th.at(0.84, th_r1 * bg * 0.85), sh.at(0.2, sh_r0 * bg * 0.85), sh.at(0.2, -sh_r0 * bg * 0.85), th.at(0.84, -th_r1 * bg * 0.85)];
                    self.seam(ctx, &kp, sc, 0.7 * al);
                }
            }
            for b in &bandages {
                let wc = self.c.get("wrap").unwrap_or([0xec, 0xe6, 0xd8]);
                match b.as_str() {
                    "calf" | "shin" => self.wraps(ctx, &sh, 0.3, 0.8, sh_r0 * bg, sh_r1 * bg, wc, 0.95 * al),
                    "thigh" => self.wraps(ctx, &th, 0.35, 0.8, th_r0 * bg, th_r1 * bg, wc, 0.95 * al),
                    _ => {}
                }
            }
            ctx.restore();
        }
        self.foot_detail(ctx, front);
    }

    /// boot soles, laces, straps, cuffs, greave plates, tabi split
    fn foot_detail(&self, ctx: &mut Ctx, front: bool) {
        let sk = &self.sk;
        let (kn, an, foot) = if front { (sk.kn_f, sk.an_f, &sk.foot_f) } else { (sk.kn_b, sk.an_b, &sk.foot_b) };
        let al = if front { 1.0 } else { 0.72 };
        let s = sk.s;
        let lk = self.leg_scale();
        let w = sk.w;
        let boot = gstr(self.style, "boots").unwrap_or("boot");
        let fw = self.sv("footwear");
        let armor = self.sv("armor");
        if fw.is_none() && !(boot == "greave" && armor.is_some()) { return; }
        let bc = if matches!(boot, "barefoot" | "wrap") { self.c.c("skin") } else { self.c.c("boot") };
        let bc = if boot == "tabi" || boot == "wrap" { self.c.get("wrap").unwrap_or([0xec, 0xe6, 0xd8]) } else { bc };
        let sh = Limb::new(kn, an);
        let (sh_r0, sh_r1) = (4.4 * w * lk, 2.9 * w * lk);
        let f = norm(sub(foot[3], foot[2]));
        let dn = perp(f);
        let on = |k: &str| fw.is_some_and(|v| gt(v, k));
        // ---- sole + heel block
        if on("sole") {
            let sc = fw.and_then(|v| gv(v, "soleColor")).map(|v| self.col_of(Some(v), [0, 0, 0])).unwrap_or(mix(bc, [0x08, 0x06, 0x0c], 0.55));
            let t = 1.15 * s;
            let sole = [foot[2], foot[3], foot[4], add(foot[4], dn, -t * 0.8), add(foot[3], dn, -t), add(foot[2], dn, -t)];
            fill_poly(ctx, &sole, sc, 0.97 * al, false);
            stroke_hi(ctx, &[vec![add(foot[2], dn, -t), add(foot[3], dn, -t), add(foot[4], dn, -t * 0.8)]], light_of(bc), 0.5 * al, 0.22);
            fill_poly(ctx, &[foot[2], lerp(foot[2], foot[3], 0.3), add(lerp(foot[2], foot[3], 0.3), dn, -t * 2.1), add(foot[2], dn, -t * 2.1), add(foot[1], f, 0.2)], sc, 0.75 * al, false);
            // tread ticks
            let tr: Vec<Vec<P>> = (0..5).map(|i| { let p = lerp(foot[3], foot[2], 0.18 + i as f64 * 0.13); vec![add(p, dn, -0.1), add(p, dn, -t * 0.75)] }).collect();
            stroke_lines(ctx, &tr, [0x00, 0x00, 0x00], 0.4 * al, 0.14);
        }
        // ---- toe cap
        if on("toeCap") {
            let a = lerp(foot[5], foot[4], 0.1);
            let c = add(lerp(foot[6], foot[3], 0.6), f, 1.2);
            stroke_lines(ctx, &[qc(add(foot[5], f, -2.6), add(c, dn, -0.2), add(foot[3], f, -2.8), 7)], shade_of(bc), 0.7 * al, 0.28);
            stroke_hi(ctx, &[qc(add(add(foot[5], f, -2.6), dn, -0.3), add(add(c, dn, -0.2), dn, -0.3), add(add(foot[3], f, -2.8), dn, -0.3), 7)], light_of(bc), 0.35 * al, 0.2);
            let _ = a;
        }
        // ---- laces over the instep
        if on("laces") {
            let ps = [foot[7], foot[6], foot[5]];
            let edge: Vec<P> = qc(ps[0], ps[1], ps[2], 8);
            let mut ticks = vec![];
            let mut eyel = vec![];
            for i in 1..6 {
                let p = edge[i + (i > 3) as usize];
                let q = add(add(p, dn, 0.9 * s), f, -0.6 * s);
                ticks.push(vec![add(p, dn, -0.3), add(q, f, 0.7)]);
                ticks.push(vec![add(p, f, 0.7), add(add(q, dn, 0.2), f, -0.1)]);
                eyel.push(p);
            }
            stroke_lines(ctx, &ticks, light_of(bc), 0.9 * al, 0.2);
            dots(ctx, &eyel, 0.2, self.c.c("accent"), 0.9 * al);
            // shaft laces: cross rungs along the front of the boot top
            if matches!(boot, "boot" | "tall" | "greave") {
                let fr = if sh.n.x * f.x + sh.n.y * f.y > 0.0 { 1.0 } else { -1.0 };
                let rungs: Vec<Vec<P>> = (0..5).flat_map(|i| {
                    let t = if boot == "tall" { 0.18 + i as f64 * 0.13 } else { 0.5 + i as f64 * 0.09 };
                    let a = sh.at(t, -fr * sh.n.x.signum() * 0.0 + fr * 0.0);
                    let _ = a;
                    let edge = |tt: f64, o: f64| add(lerp(kn, an, tt), sh.n, fr * o);
                    vec![vec![edge(t, sh_r0 * 0.15), edge(t + 0.06, sh_r1 * 0.85)], vec![edge(t, sh_r1 * 0.85), edge(t + 0.06, sh_r0 * 0.15)]]
                }).collect();
                let mut rg = rungs;
                if !matches!(boot, "tall") { rg.truncate(8); }
                ctx.save();
                clip_geo(ctx, &[Geo::Capsule(lerp(kn, an, 0.05), an, sh_r0 * 1.12, sh_r1 * 1.35)]);
                stroke_lines(ctx, &rg, light_of(bc), 0.7 * al, 0.2);
                ctx.restore();
            }
        }
        // ---- straps / buckles / cuff
        let nb = fw.map(|v| gnum(v, "buckles", 0.0)).unwrap_or(0.0) as i32;
        if nb > 0 {
            let ac = self.c.c("accent");
            ctx.save();
            clip_geo(ctx, &[Geo::Capsule(lerp(kn, an, 0.05), an, sh_r0 * 1.14, sh_r1 * 1.38)]);
            for i in 0..nb {
                let t = 0.25 + (0.5 / nb.max(1) as f64) * i as f64 + 0.1;
                let r = sh_r0 * 1.4;
                fill_poly(ctx, &[sh.at(t, -r), sh.at(t, r), sh.at(t + 0.07, r), sh.at(t + 0.07, -r)], mix(bc, [0xff, 0xf0, 0xe0], 0.18), 0.95 * al, false);
                stroke_lines(ctx, &[vec![sh.at(t, -r), sh.at(t, r)], vec![sh.at(t + 0.07, -r), sh.at(t + 0.07, r)]], shade_of(bc), 0.7 * al, 0.2);
                let b = sh.at(t + 0.035, sh_r1 * 0.9);
                fill_ell(ctx, b, 0.9, 0.8, ang(sh.u), ac, 0.97 * al);
                fill_ell(ctx, b, 0.45, 0.4, ang(sh.u), shade_of(bc), 0.9 * al);
            }
            ctx.restore();
        }
        if on("cuff") {
            let top = if boot == "tall" { 0.06 } else { 0.42 };
            let r = sh_r0 * 1.2;
            fill_poly(ctx, &[sh.at(top, -r), sh.at(top, r), sh.at(top + 0.07, r * 0.98), sh.at(top + 0.07, -r * 0.98)], light_of(bc), 0.35 * al, false);
            stroke_lines(ctx, &[vec![sh.at(top + 0.07, -r * 0.98), sh.at(top + 0.07, r * 0.98)]], shade_of(bc), 0.7 * al, 0.26);
        }
        // ---- tabi split toe / wraps cross
        if on("split") {
            stroke_lines(ctx, &[vec![lerp(foot[5], foot[4], 0.5), add(foot[6], f, 2.6)]], shade_of(bc), 0.8 * al, 0.2);
        }
        if on("wrapped") {
            let wc = self.c.get("wrap").unwrap_or([0xec, 0xe6, 0xd8]);
            let l = Limb::new(lerp(kn, an, 0.55), an);
            ctx.save();
            clip_geo(ctx, &[Geo::Capsule(lerp(kn, an, 0.55), an, sh_r0 * 0.9, sh_r1 * 1.3)]);
            self.wraps(ctx, &l, 0.0, 1.0, sh_r0 * 0.75, sh_r1 * 1.0, wc, 0.95 * al);
            ctx.restore();
        }
        // ---- greaves: lit ridge, plate seams, rivets
        if boot == "greave" && armor.is_some() {
            let m = self.c.c("metal");
            let nn = norm(perp(sub(an, kn)));
            let _ = nn;
            ctx.save();
            clip_geo(ctx, &[Geo::Capsule(lerp(kn, an, 0.25), an, sh_r0 * 1.12, sh_r1 * 1.35)]);
            let ridge: Vec<P> = (0..=8).map(|i| sh.at(0.28 + 0.62 * i as f64 / 8.0, sh_r0 * 0.4 * (1.0 - i as f64 / 14.0))).collect();
            stroke_lines(ctx, &[ridge], light_of(m), 0.6 * al, 0.36);
            let seams: Vec<Vec<P>> = [0.42, 0.6, 0.78].iter().map(|t| vec![sh.at(*t, -sh_r0 * 1.4), sh.at(*t + 0.015, 0.0), sh.at(*t, sh_r0 * 1.4)]).collect();
            stroke_lines(ctx, &seams, shade_of(m), 0.6 * al, 0.3);
            let sh2: Vec<Vec<P>> = seams.iter().map(|l| l.iter().map(|p| add(*p, sh.u, 0.35)).collect()).collect();
            stroke_hi(ctx, &sh2, light_of(m), 0.3 * al, 0.2);
            let rv: Vec<P> = [0.35, 0.69].iter().map(|t| sh.at(*t, -sh_r0 * 0.5)).collect();
            dots(ctx, &rv, 0.4, shade_of(m), 0.85 * al);
            ctx.restore();
        }
    }
}

fn skin_tint() -> Rgb { [0xc0, 0x90, 0x80] }

// ------------------------------------------------------------------ fabric tiles
thread_local! { static TILES: std::cell::RefCell<HashMap<(String, Rgb, u32), Option<Rc<Pixmap>>>> = std::cell::RefCell::new(HashMap::new()); }

fn fabric_tile(kind: &str, base: Rgb) -> Option<Rc<Pixmap>> {
    let key = (kind.to_string(), base, (res() * 100.0) as u32);
    TILES.with(|t| t.borrow_mut().entry(key).or_insert_with(|| make_tile(kind, base)).clone())
}

/// one period of the pattern, drawn in unit space (1 unit = `res()` px); each stroke is repeated on the
/// neighbouring periods so the tile wraps seamlessly
fn make_tile(kind: &str, base: Rgb) -> Option<Rc<Pixmap>> {
    let (sh, hi) = (shade_of(base), light_of(base));
    let (px, py) = match kind { "hatch" => (1.1, 1.1), "weave" => (0.85, 0.85), "check" => (3.2, 3.2), "rib" => (1.0, 1.0), "stripes" => (1.0, 3.4), "scales" => (2.3, 2.3), "chain" => (1.15, 1.15), "quilt" => (2.8, 2.8), "dots" => (2.0, 1.6), _ => return None };
    let r = res();
    let (tw, th) = ((px * r).round().max(2.0), (py * r).round().max(2.0));
    let (px, py) = (tw / r, th / r);
    let mut ctx = Ctx::new(tw as u32, th as u32)?;
    ctx.scale(r, r);
    let wrap = |ls: Vec<Vec<P>>| -> Vec<Vec<P>> {
        let mut out = vec![];
        for dx in [-1.0, 0.0, 1.0] { for dy in [-1.0, 0.0, 1.0] { for l in &ls { out.push(l.iter().map(|p| pt(p.x + dx * px, p.y + dy * py)).collect()); } } }
        out
    };
    match kind {
        "hatch" => stroke_lines(&mut ctx, &wrap(vec![vec![pt(0.0, py), pt(px, 0.0)]]), sh, 0.16, 0.28),
        "weave" => { stroke_lines(&mut ctx, &wrap(vec![vec![pt(0.0, 0.0), pt(0.0, py)]]), sh, 0.1, 0.22); stroke_lines(&mut ctx, &wrap(vec![vec![pt(0.0, 0.0), pt(px, 0.0)]]), sh, 0.07, 0.22); }
        "check" => { stroke_lines(&mut ctx, &wrap(vec![vec![pt(0.0, 0.0), pt(0.0, py)]]), sh, 0.2, 0.9); stroke_lines(&mut ctx, &wrap(vec![vec![pt(0.0, 0.0), pt(px, 0.0)]]), hi, 0.18, 0.9); }
        "rib" => { stroke_lines(&mut ctx, &wrap(vec![vec![pt(0.0, 0.0), pt(px, 0.0)]]), sh, 0.2, 0.3); stroke_lines(&mut ctx, &wrap(vec![vec![pt(0.0, 0.45), pt(px, 0.45)]]), hi, 0.12, 0.25); }
        "stripes" => fill_poly(&mut ctx, &[pt(0.0, 0.0), pt(px, 0.0), pt(px, 1.3), pt(0.0, 1.3)], sh, 0.22, false),
        "scales" | "chain" => {
            let rad = if kind == "scales" { 1.15 } else { 0.5 };
            let arc = |c: P| -> Vec<P> { (0..=6).map(|i| { let t = PI * 0.15 + i as f64 / 6.0 * PI * 0.7; pt(c.x + t.sin() * rad, c.y + t.cos() * rad) }).collect() };
            let mut ls = vec![];
            for (cx, cy) in [(0.0, 0.0), (px * 0.5, py * 0.5)] { ls.push(arc(pt(cx, cy))); }
            let ls = wrap(ls);
            stroke_lines(&mut ctx, &ls, sh, 0.34, if kind == "scales" { 0.3 } else { 0.18 });
            let hs: Vec<Vec<P>> = ls.iter().map(|l| l.iter().map(|p| add(*p, light(), 0.2)).collect()).collect();
            stroke_lines(&mut ctx, &hs, hi, 0.2, 0.16);
        }
        "quilt" => {
            let d = wrap(vec![vec![pt(0.0, py), pt(px, 0.0)], vec![pt(0.0, 0.0), pt(px, py)]]);
            stroke_lines(&mut ctx, &d, sh, 0.28, 0.3);
            let h: Vec<Vec<P>> = d.iter().map(|l| l.iter().map(|p| add(*p, light(), 0.3)).collect()).collect();
            stroke_lines(&mut ctx, &h, hi, 0.22, 0.22);
        }
        _ => dots(&mut ctx, &[pt(px * 0.5, py * 0.5)], 0.28, sh, 0.3),
    }
    Some(Rc::new(ctx.pm))
}
