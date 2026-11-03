//! Torso / belt / cloth / pauldron detail (see `detail.rs`).
use super::detail::*;
use super::*;

impl<'a> Rig<'a> {
    pub(super) fn any_body_detail(&self) -> bool {
        ["muscle", "folds", "stitch", "weave", "emblem", "sash", "necklace", "armor", "tattoo", "veins", "scars", "bandages"].iter().any(|k| self.sv(k).is_some())
    }

    /// visual-only torso outline (chest / waist multipliers); identical to the skeleton's when absent
    pub(super) fn vis_torso(sk: &Skel, style: &Value) -> Vec<P> {
        let (ch, wa) = (gnum(style, "chest", 1.0), gnum(style, "waist", 1.0));
        if ch == 1.0 && wa == 1.0 { return sk.torso_pts.clone(); }
        let tl = torso_local(sk.w, sk.fem, sk.l.torso, gstr(style, "torsoShape").unwrap_or(""));
        let k = sk.l.torso / 29.0;
        tl.into_iter().map(|(v, u)| {
            let f = u / k;
            let sm = |a: f64, b: f64| { let t = ((f - a) / (b - a)).clamp(0.0, 1.0); t * t * (3.0 - 2.0 * t) };
            let m = 1.0 + (ch - 1.0) * sm(10.0, 22.0) + (wa - 1.0) * (1.0 - sm(2.0, 14.0)) * sm(-4.0, 3.0);
            add(add(sk.hip, sk.fwd, v * m), sk.up, u)
        }).collect()
    }

    fn q(&self, v: f64, u: f64) -> P { self.t(v * self.sk.w, u * self.sk.l.torso / 29.0) }

    /// pecs, abs, serratus, clavicles, back; `base` is the colour being shaped (skin or stretched cloth)
    fn muscle_torso(&self, ctx: &mut Ctx, base: Rgb, k: f64, full: bool, back: bool) {
        let (sk, w) = (&self.sk, self.sk.w);
        let tl = sk.l.torso / 29.0;
        let q = |v: f64, u: f64| self.q(v, u);
        let (hi, shd) = (mix(base, [0xff, 0xf6, 0xe8], 0.5), mix(base, [0x2b, 0x17, 0x48], 0.5));
        let k = k.min(1.4);
        let fr = ang(sk.fwd);
        // pecs
        stroke_lines(ctx, &[qc(q(1.5, 22.0), q(5.0, 16.6), q(8.9, 19.4), 9)], shd, 0.34 * k, 1.1 * tl);
        fill_ell(ctx, q(5.7, 22.4), 2.9 * w, 1.5 * tl, fr - 0.25, hi, 0.3 * k);
        stroke_lines(ctx, &[vec![q(1.9, 24.0), q(2.2, 21.0), q(2.0, 18.5)]], shd, 0.4 * k, 0.4);
        // clavicle + trapezius slope + deltoid edge
        stroke_lines(ctx, &[qc(q(0.5, 28.2), q(3.2, 27.2), q(7.8, 27.6), 7), qc(q(-1.8, 30.8), q(-0.9, 29.4), q(0.3, 28.3), 5), qc(q(8.7, 26.0), q(9.8, 22.0), q(8.8, 19.6), 6)], shd, 0.42 * k, 0.34);
        if !full { return; }
        // abs: highlight on each block + shade line under it
        let mut hl = vec![];
        let mut sl = vec![];
        for (r, u) in [13.4, 9.4, 5.4].iter().enumerate() {
            for (c, (v0, v1)) in [(2.5, 4.3), (4.7, 6.5)].iter().enumerate() {
                let _ = (r, c);
                hl.push((q((v0 + v1) * 0.5, u + 0.6), (v1 - v0) * 0.5 * w));
                sl.push(qc(q(*v0, u - 1.7), q((v0 + v1) * 0.5, u - 2.1), q(*v1, u - 1.7), 5));
            }
        }
        for (c, r) in hl { fill_ell(ctx, c, r * 0.9, 0.85 * tl, fr, hi, 0.26 * k); }
        stroke_lines(ctx, &sl, shd, 0.46 * k, 0.4);
        stroke_lines(ctx, &[vec![q(4.5, 16.0), q(4.55, 9.0), q(4.5, 2.6)]], shd, 0.46 * k, 0.34);
        // serratus + oblique
        let ser: Vec<Vec<P>> = (0..3).map(|i| { let u = 18.0 - i as f64 * 2.3; vec![q(8.2, u), q(6.8, u - 2.0)] }).collect();
        stroke_lines(ctx, &ser, shd, 0.42 * k, 0.3);
        stroke_lines(ctx, &[qc(q(8.9, 12.5), q(8.6, 7.5), q(7.2, 3.5), 6)], shd, 0.34 * k, 0.4);
        if back {
            fill_ell(ctx, q(-4.6, 23.0), 2.5 * w, 2.2 * tl, fr + 0.4, hi, 0.2 * k);
            stroke_lines(ctx, &[qc(q(-6.4, 26.0), q(-5.2, 15.0), q(-7.2, 6.0), 7), vec![q(-1.6, 28.0), q(-1.8, 18.0)]], shd, 0.34 * k, 0.4);
            stroke_lines(ctx, &[qc(q(-2.0, 12.0), q(-4.0, 11.0), q(-6.5, 12.5), 5)], shd, 0.28 * k, 0.3);
        }
        if self.sv("veins").is_some() {
            let vc = mix(base, [0x4a, 0x5a, 0xa8], 0.42);
            let v1: Vec<P> = (0..=8).map(|i| { let t = i as f64 / 8.0; q(7.3 + (t * 12.0).sin() * 0.35, 4.0 + t * 8.5) }).collect();
            stroke_lines(ctx, &[v1, qc(q(8.4, 17.6), q(7.8, 19.0), q(7.2, 18.6), 5)], vc, 0.5, 0.2);
        }
    }

    pub(super) fn draw_body_detail(&self, ctx: &mut Ctx) {
        let (sk, c, style) = (&self.sk, &self.c, self.style);
        let (w, s) = (sk.w, sk.s);
        let _tl = sk.l.torso / 29.0;
        let q = |v: f64, u: f64| self.q(v, u);
        let skin = c.c("skin");
        let top_c = c.c("top");
        let ol = c.c("outline");
        let top = self.top.as_str();
        let m = self.sn("muscle", 0.0) * 1.3;
        let folds = self.sn("folds", 0.0).min(1.6);
        let weave = self.ss("weave").unwrap_or("");
        let cloth_top = !matches!(top, "bare" | "armor" | "suit");
        let fr = ang(sk.fwd);
        // ---- muscle (skin areas)
        if m > 0.0 && !sk.fem || (m > 0.0 && top == "bare") {
            match top {
                "bare" => self.muscle_torso(ctx, skin, m, true, true),
                "vest" => {
                    ctx.save();
                    clip_geo(ctx, &[Geo::Poly(vec![q(1.0, 30.0), q(9.6, 26.0), q(9.6, 3.0), q(4.5, 3.0)], false)]);
                    self.muscle_torso(ctx, skin, m, true, false);
                    ctx.restore();
                }
                "gi" | "robe" => {
                    ctx.save();
                    clip_geo(ctx, &[Geo::Poly(vec![q(-1.2, 32.0), q(6.5, 31.0), q(3.5, 12.0)], false)]);
                    self.muscle_torso(ctx, skin, m, false, false);
                    ctx.restore();
                }
                "tank" => self.muscle_torso(ctx, top_c, m * 0.6, false, false),
                _ => {}
            }
        }
        // ---- fabric weave
        if cloth_top && !weave.is_empty() {
            let o = add(add(sk.hip, sk.fwd, -10.5 * w), sk.up, -3.0);
            self.weave(ctx, weave, if top == "jacket" || top == "coat" { c.c("top2") } else { top_c }, &[Geo::Poly(self.vt.clone(), true)], o, sk.up, 1.0);
        }
        // ---- cloth folds
        if cloth_top && folds > 0.0 {
            let f = folds.min(1.0);
            let seed = self.sn("foldSeed", 0.0);
            let n = 5 + (folds * 3.0) as i32;
            let mut dk = vec![];
            let mut lt = vec![];
            for i in 0..n {
                let h = hash(i as f64 * 3.1 + seed);
                let h2 = hash(i as f64 * 7.7 + seed + 2.0);
                let v0 = -7.5 + (i as f64 + h * 0.6) * (15.5 / n as f64);
                let a = q(v0, 3.0 + h2 * 4.0);
                let b = q(v0 - 1.2 - h * 2.2, 13.0 + h2 * 11.0);
                let cc = q(v0 + 1.6 * (h - 0.5) * 2.0 - 0.8, 8.0 + h2 * 6.0);
                let pl = qc(a, cc, b, 7);
                lt.push(pl.iter().map(|p| add(*p, light(), 0.55)).collect::<Vec<P>>());
                dk.push(pl);
            }
            stroke_lines(ctx, &dk, shade_of(top_c), 0.5 * f, 0.42);
            stroke_hi(ctx, &lt, light_of(top_c), 0.22 * f, 0.3);
            // armpit tension fan
            let ap = q(2.2, 24.0);
            let fan: Vec<Vec<P>> = [(6.5, 19.0), (5.2, 15.5), (3.0, 13.0), (0.5, 14.5)].iter().map(|&(v, u)| qc(ap, lerp(ap, q(v, u), 0.5), q(v, u), 5)).collect();
            stroke_lines(ctx, &fan, shade_of(top_c), 0.36 * f, 0.34);
            // belly compression ripples above the belt
            let rip: Vec<Vec<P>> = (0..3).map(|i| { let u = 6.5 + i as f64 * 1.7; qc(q(-7.0, u), q(0.5, u + 1.1 - i as f64 * 0.3), q(8.5, u - 0.2), 8) }).collect();
            stroke_lines(ctx, &rip, shade_of(top_c), 0.32 * f, 0.34);
            fill_poly(ctx, &[q(-7.0, 0.0), q(8.8, 0.0), q(8.8, 5.5), q(-7.0, 7.5)], shade_of(top_c), 0.14 * f, true);
            // back side shading
            fill_poly(ctx, &[q(-9.0, 30.0), q(-6.5, 28.0), q(-7.3, 6.0), q(-9.5, 2.0)], shade_of(top_c), 0.2 * f, true);
        }
        // ---- stitching
        if let Some(sc) = self.stitch_col(top_c) {
            if cloth_top {
                let seam = |ctx: &mut Ctx, pts: Vec<P>| self.seam(ctx, &pts, sc, 0.8);
                seam(ctx, qc(q(-6.8, 27.0), q(-7.6, 16.0), q(-6.9, 5.5), 8));
                seam(ctx, qc(q(-3.4, 31.0), q(0.0, 28.5), q(2.6, 25.0), 6));
                seam(ctx, qc(q(2.6, 24.0), q(0.4, 17.0), q(-0.6, 9.0), 7));
                seam(ctx, vec![q(-8.4, 1.6), q(0.0, 1.9), q(8.8, 1.4)]);
                if matches!(top, "gi" | "robe" | "jacket" | "coat") {
                    let off = |p: P| add(p, sk.fwd, 1.1);
                    seam(ctx, [q(-0.5, 32.0), q(1.5, 22.0), q(4.2, 6.0), q(4.4, -3.0)].iter().map(|p| off(*p)).collect());
                }
            }
        }
        // ---- armor plating
        if let Some(a) = self.sv("armor") {
            if matches!(top, "armor" | "suit") { self.armor_torso(ctx, a); }
        }
        // ---- emblem, sash, necklace, tattoo, scars, bandages (torso)
        if let Some(e) = self.sv("emblem") {
            let back = gstr(e, "where") == Some("back");
            let sz = gnum(e, "size", 1.0) * 3.0 * s;
            let pos = if back { q(-3.6, 19.0) } else { q(5.2, 18.5) };
            let col = self.col_of(gv(e, "color"), c.c("accent"));
            self.emblem(ctx, gstr(e, "type").unwrap_or("star"), pos, sz, col, fr, if back { 0.85 } else { 1.0 });
        }
        if let Some(sa) = self.sv("sash") {
            let col = self.col_of(gv(sa, "color"), c.c("belt"));
                        let (a, b) = (q(-4.5, 32.0), q(8.8, 3.0));
            let (a2, b2) = (q(-1.5, 32.5), q(10.6, 3.6));
            let strap = [a, a2, b2, b];
            fill_poly(ctx, &strap, col, 1.0, false);
            let nrm = norm(perp(sub(b, a)));
            stroke_lines(ctx, &[vec![a, b], vec![a2, b2]], mix(col, [0, 0, 0], 0.45), 0.9, 0.5);
            stroke_hi(ctx, &[vec![add(a, light(), 0.5), add(b, light(), 0.5)]], light_of(col), 0.3, 0.3);
            let mid_a = lerp(a, a2, 0.5);
            let mid_b = lerp(b, b2, 0.5);
            self.seam(ctx, &[lerp(mid_a, mid_b, 0.04), lerp(mid_a, mid_b, 0.96)], light_of(col), 0.7);
            if gt(sa, "rounds") {
                let rd: Vec<Vec<P>> = (0..7).map(|i| { let t = 0.12 + i as f64 * 0.12; let p = lerp(mid_a, mid_b, t); vec![add(p, nrm, -0.9), add(p, nrm, 0.9)] }).collect();
                stroke_lines(ctx, &rd, c.c("accent"), 0.95, 0.75);
                stroke_hi(ctx, &rd.iter().map(|l| vec![add(l[0], light(), 0.2), add(l[1], light(), 0.2)]).collect::<Vec<_>>(), WHITE, 0.45, 0.2);
            }
            if gt(sa, "buckle") {
                let p = lerp(mid_a, mid_b, 0.5);
                fill_ell(ctx, p, 1.7 * s, 1.5 * s, ang(sub(b, a)), c.c("accent"), 1.0);
                fill_ell(ctx, p, 0.9 * s, 0.8 * s, ang(sub(b, a)), shade_of(col), 1.0);
            }
        }
        if let Some(n) = self.sv("necklace") { self.necklace(ctx, n); }
        if let Some(sc) = gv(style, "scars").and_then(|v| v.as_array()) {
            let sc_col = mix(skin, [0xa0, 0x30, 0x40], 0.7);
            for part in sc.iter().filter_map(|x| x.as_str()) {
                if part == "chest" && !cloth_top {
                    let (a, b) = (q(2.0, 25.0), q(8.0, 17.0));
                    stroke_lines(ctx, &[vec![a, b]], sc_col, 0.9, 0.45);
                    let nrm = norm(perp(sub(b, a)));
                    let ticks: Vec<Vec<P>> = (1..5).map(|i| { let p = lerp(a, b, i as f64 / 5.0); vec![add(p, nrm, -0.7), add(p, nrm, 0.7)] }).collect();
                    stroke_lines(ctx, &ticks, sc_col, 0.8, 0.2);
                }
                if part == "ribs" && !cloth_top {
                    let ls: Vec<Vec<P>> = (0..3).map(|i| vec![q(-6.5, 10.0 + i as f64 * 2.0), q(-3.5, 9.0 + i as f64 * 2.0)]).collect();
                    stroke_lines(ctx, &ls, sc_col, 0.85, 0.35);
                }
            }
        }
        if let Some(t) = self.sv("tattoo") {
            let sites = gv(t, "on").and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>()).unwrap_or_default();
            let tc = self.col_of(gv(t, "color"), mix(skin, [0x28, 0x2a, 0x60], 0.7));
            if sites.contains(&"chest") && !cloth_top { self.emblem(ctx, gstr(t, "chestType").unwrap_or("flame"), q(5.4, 19.5), 3.4 * s, tc, fr, 0.85); }
            if sites.contains(&"back") && !cloth_top { self.emblem(ctx, gstr(t, "backType").unwrap_or("lotus"), q(-3.8, 19.0), 4.6 * s, tc, fr, 0.8); }
            if sites.contains(&"ribs") && !cloth_top {
                let ls: Vec<Vec<P>> = (0..4).map(|i| qc(q(7.8, 15.0 - i as f64 * 1.6), q(8.6, 14.0 - i as f64 * 1.6), q(7.0, 13.2 - i as f64 * 1.6), 4)).collect();
                stroke_lines(ctx, &ls, tc, 0.85, 0.5);
            }
        }
        if self.bandage_list().iter().any(|b| b == "chest") {
            let wc = c.get("wrap").unwrap_or([0xec, 0xe6, 0xd8]);
            let band = [q(-9.5, 17.0), q(10.0, 19.0), q(10.0, 12.0), q(-9.5, 10.0)];
            fill_poly(ctx, &band, wc, 0.96, false);
            let dk: Vec<Vec<P>> = (0..9).map(|i| { let v = -9.0 + i as f64 * 2.1; vec![q(v, 10.5 + i as f64 * 0.25), q(v + 1.5, 17.5 + i as f64 * 0.25)] }).collect();
            stroke_lines(ctx, &dk, shade_of(wc), 0.7, 0.3);
            stroke_hi(ctx, &dk.iter().map(|l| vec![add(l[0], light(), 0.3), add(l[1], light(), 0.3)]).collect::<Vec<_>>(), WHITE, 0.4, 0.2);
        }
        let _ = ol;
    }

    fn armor_torso(&self, ctx: &mut Ctx, a: &Value) {
        let (sk, c) = (&self.sk, &self.c);
        let (w, s) = (sk.w, sk.s);
        let q = |v: f64, u: f64| self.q(v, u);
        let m = c.c("metal");
        let (hi, shd) = (light_of(m), shade_of(m));
        let pp = [q(-6.0, 27.0), q(-1.0, 30.0), q(8.0, 27.0), q(9.4, 18.0), q(6.0, 12.0), q(-6.0, 14.0)];
        // bevel: lit upper rim, shadowed lower rim, inset panel
        stroke_lines(ctx, &[vec![add(pp[0], light(), -0.3), add(pp[1], light(), -0.3), add(pp[2], light(), -0.3), add(pp[3], light(), -0.3)]], hi, 0.85, 0.55);
        stroke_lines(ctx, &[vec![pp[3], pp[4], pp[5]]], shd, 0.8, 0.8);
        let cen = lerp(pp[1], pp[4], 0.5);
        let inset: Vec<P> = pp.iter().map(|p| lerp(*p, cen, 0.17)).collect();
        stroke_poly(ctx, &inset, shd, 0.55, 0.3, true);
        stroke_hi(ctx, &[vec![add(inset[0], light(), 0.25), add(inset[1], light(), 0.25), add(inset[2], light(), 0.25)]], hi, 0.35, 0.2);
        // centre ridge
        stroke_lines(ctx, &[vec![q(1.6, 29.0), q(2.4, 13.4)]], shd, 0.7, 0.45);
        stroke_lines(ctx, &[vec![q(1.6 + 0.45, 29.0), q(2.4 + 0.45, 13.4)]], hi, 0.5, 0.25);
        // lames below the breastplate
        for (i, u) in [11.0, 8.0, 5.0].iter().enumerate() {
            let ls = [q(-6.4, *u), q(1.0, u - 0.8), q(8.0, *u)];
            fill_poly(ctx, &[ls[0], ls[1], ls[2], q(8.0, u - 1.4), q(1.0, u - 2.2), q(-6.4, u - 1.4)], mix(m, [0x00, 0x00, 0x08], 0.12 + i as f64 * 0.05), 0.8, true);
            stroke_hi(ctx, &[qc(ls[0], ls[1], ls[2], 6)], hi, 0.6, 0.3);
            stroke_lines(ctx, &[qc(add(ls[0], light(), -0.8), add(ls[1], light(), -0.8), add(ls[2], light(), -0.8), 6)], shd, 0.7, 0.4);
        }
        // engraving
        let eng = gstr(a, "engrave").unwrap_or("");
        let ec = self.col_of(gv(a, "engraveColor"), c.c("accent"));
        match eng {
            "scroll" => {
                for (cx, sg) in [(q(2.0, 21.0), 1.0), (q(5.6, 21.0), -1.0)] {
                    let ls: Vec<P> = (0..=24).map(|i| { let t = i as f64 * 0.34; let r = (0.4 + i as f64 * 0.07) * s; pt(cx.x + sg * t.cos() * r * 1.1, cx.y + t.sin() * r * 0.9) }).collect();
                    stroke_lines(ctx, &[ls], ec, 0.85, 0.28);
                }
                stroke_lines(ctx, &[qc(q(0.0, 17.0), q(3.8, 15.0), q(7.6, 17.0), 8)], ec, 0.8, 0.3);
            }
            "chevron" => {
                let ls: Vec<Vec<P>> = (0..3).map(|i| { let u = 24.0 - i as f64 * 2.4; vec![q(-3.0, u), q(2.0, u - 2.6), q(7.0, u)] }).collect();
                stroke_lines(ctx, &ls, ec, 0.85, 0.35);
            }
            "circuit" => {
                let ls = vec![vec![q(-4.0, 25.0), q(-4.0, 21.0), q(-1.0, 21.0), q(-1.0, 17.0)], vec![q(7.0, 25.0), q(7.0, 22.5), q(8.5, 22.5)], vec![q(-2.0, 25.0), q(1.0, 25.0), q(1.0, 22.5)], vec![q(2.0, 14.0), q(5.0, 14.0), q(5.0, 16.5)]];
                stroke_lines(ctx, &ls, ec, 0.85, 0.28);
                let nodes: Vec<P> = ls.iter().filter_map(|l| l.last().copied()).collect();
                dots(ctx, &nodes, 0.5, ec, 0.95);
            }
            "spikes" => {
                for i in 0..5 { let v = -3.5 + i as f64 * 2.6; fill_poly(ctx, &[q(v, 14.4), q(v + 1.0, 17.8), q(v + 2.0, 14.4)], ec, 0.8, false); }
            }
            _ => {}
        }
        // rivets
        if gt(a, "rivets") {
            let rv: Vec<P> = vec![q(-5.0, 26.0), q(7.0, 25.8), q(8.4, 18.5), q(5.6, 13.4), q(-5.0, 14.6), q(-5.2, 20.5), q(1.6, 29.5)];
            dots(ctx, &rv, 0.5 * s.max(0.9), shd, 0.95);
            let rh: Vec<P> = rv.iter().map(|p| add(*p, light(), 0.14)).collect();
            dots(ctx, &rh, 0.26 * s.max(0.9), WHITE, 0.75);
            let _ = w;
        }
    }

    fn necklace(&self, ctx: &mut Ctx, n: &Value) {
        let (sk, c) = (&self.sk, &self.c);
        let s = sk.s;
        let q = |v: f64, u: f64| self.q(v, u);
        let kind = gstr(n, "type").unwrap_or("beads");
        let col = self.col_of(gv(n, "color"), c.c("accent"));
        let a = q(-1.6, 31.5);
        let b = q(7.2, 29.5);
        let low = q(3.6, 22.5);
        let arc = qc(a, low, b, 14);
        match kind {
            "beads" => {
                let ps: Vec<P> = (0..=12).map(|i| arc[i]).collect();
                dots(ctx, &ps, 0.72 * s, shade_of(col), 0.95);
                dots(ctx, &ps.iter().map(|p| add(*p, light(), 0.12)).collect::<Vec<_>>(), 0.5 * s, col, 1.0);
                dots(ctx, &ps.iter().map(|p| add(*p, light(), 0.3)).collect::<Vec<_>>(), 0.16 * s, WHITE, 0.7);
                let m = arc[7];
                fill_ell(ctx, add(m, pt(0.0, 1.2), 1.0), 1.3 * s, 1.1 * s, 0.0, shade_of(col), 1.0);
                fill_ell(ctx, add(m, pt(0.0, 1.2), 1.0), 0.9 * s, 0.75 * s, 0.0, light_of(col), 1.0);
            }
            "tag" => {
                stroke_lines(ctx, &[arc.clone()], mix(c.c("metal"), [0, 0, 0], 0.2), 0.95, 0.35);
                let m = arc[7];
                for k in 0..2 {
                    let p = add(m, pt(0.4 * k as f64, 1.6 + 0.5 * k as f64), 1.0);
                    fill_poly(ctx, &[add(p, pt(-1.0, -1.4), 1.0), add(p, pt(1.0, -1.4), 1.0), add(p, pt(1.2, 1.2), 1.0), add(p, pt(-1.2, 1.2), 1.0)], mix(c.c("metal"), WHITE, 0.2 + 0.15 * k as f64), 1.0, false);
                    stroke_lines(ctx, &[vec![add(p, pt(-0.6, -0.2), 1.0), add(p, pt(0.6, -0.2), 1.0)], vec![add(p, pt(-0.6, 0.5), 1.0), add(p, pt(0.5, 0.5), 1.0)]], [0x30, 0x30, 0x38], 0.8, 0.14);
                }
            }
            "chain" | "pendant" => {
                stroke_lines(ctx, &[arc.clone()], col, 0.95, 0.32);
                stroke_lines(ctx, &[arc.iter().map(|p| add(*p, light(), 0.3)).collect::<Vec<_>>()], WHITE, 0.4, 0.14);
                let m = arc[7];
                let p = add(m, pt(0.0, 1.8), 1.0);
                fill_poly(ctx, &[add(p, pt(0.0, -1.4), 1.0), add(p, pt(1.2, 0.0), 1.0), add(p, pt(0.0, 1.6), 1.0), add(p, pt(-1.2, 0.0), 1.0)], col, 1.0, false);
                fill_poly(ctx, &[add(p, pt(0.0, -0.8), 1.0), add(p, pt(0.5, 0.0), 1.0), add(p, pt(0.0, 0.8), 1.0), add(p, pt(-0.5, 0.0), 1.0)], self.col_of(gv(n, "gem"), c.c("energy")), 1.0, false);
            }
            _ => {}
        }
    }

    fn emblem(&self, ctx: &mut Ctx, kind: &str, p: P, r: f64, col: Rgb, rot: f64, a: f64) {
        let sh = shade_of(col);
        let at = |x: f64, y: f64| { let (sn, cs) = rot.sin_cos(); pt(p.x + (x * cs - y * sn) * r, p.y + (x * sn + y * cs) * r) };
        let up = |x: f64, y: f64| at(x, y - 0.0);
        match kind {
            "circle" => {
                ctx.begin(); ctx.move_to(p.x + r, p.y); ctx.arc(p.x, p.y, r, 0.0, PI * 2.0); ctx.stroke_w(&sol(col, a), 0.6);
                fill_ell(ctx, p, r * 0.42, r * 0.42, 0.0, col, a);
            }
            "bolt" => fill_poly(ctx, &[up(0.2, -1.0), up(-0.6, 0.1), up(-0.05, 0.1), up(-0.35, 1.0), up(0.6, -0.2), up(0.05, -0.2), up(0.5, -1.0)], col, a, false),
            "moon" => {
                let c1: Vec<P> = (0..=14).map(|i| { let t = PI * 0.5 + i as f64 / 14.0 * PI; at(t.cos(), t.sin()) }).collect();
                let c2: Vec<P> = (0..=14).rev().map(|i| { let t = PI * 0.5 + i as f64 / 14.0 * PI; at(t.cos() * 0.55 + 0.35, t.sin() * 0.9) }).collect();
                let mut pts = c1; pts.extend(c2);
                fill_poly(ctx, &pts, col, a, false);
            }
            "lotus" => {
                for k in 0..5 {
                    let ar = (k as f64 - 2.0) * 0.5 - PI / 2.0 + rot;
                    fill_ell(ctx, pt(p.x + ar.cos() * r * 0.55, p.y + ar.sin() * r * 0.55), r * 0.62, r * 0.26, ar, col, a * 0.92);
                }
                fill_ell(ctx, p, r * 0.22, r * 0.22, 0.0, light_of(col), a);
            }
            "glyph" => {
                let mut ls = vec![vec![at(-0.7, -0.7), at(0.7, -0.7), at(0.7, 0.7), at(-0.7, 0.7), at(-0.7, -0.7)]];
                for i in 0..4 { let h = hash(i as f64 * 5.3 + 1.0); let h2 = hash(i as f64 * 2.9 + 4.0); ls.push(vec![at(-0.5 + h * 0.4, -0.5 + i as f64 * 0.3), at(0.1 + h2 * 0.5, -0.5 + i as f64 * 0.3 + (h - 0.5) * 0.3)]); }
                ls.push(vec![at(0.0, -0.7), at(0.05, 0.7)]);
                stroke_lines(ctx, &ls, col, a, 0.4);
            }
            "crest" => {
                fill_poly(ctx, &[at(-0.8, -0.9), at(0.8, -0.9), at(0.8, 0.1), at(0.0, 1.0), at(-0.8, 0.1)], col, a, false);
                fill_poly(ctx, &[at(-0.5, -0.6), at(0.5, -0.6), at(0.5, 0.0), at(0.0, 0.65), at(-0.5, 0.0)], sh, a * 0.9, false);
                fill_poly(ctx, &star_pts(at(0.0, -0.05), r * 0.32, rot), light_of(col), a, false);
            }
            "claw" => {
                for i in 0..3 { let o = (i as f64 - 1.0) * 0.55; fill_poly(ctx, &[at(-0.5 + o, -1.0), at(-0.25 + o, -0.9), at(0.4 + o, 1.0), at(0.15 + o, 0.85)], col, a, false); }
            }
            "diamond" => {
                fill_poly(ctx, &[at(0.0, -1.0), at(0.75, 0.0), at(0.0, 1.0), at(-0.75, 0.0)], col, a, false);
                fill_poly(ctx, &[at(0.0, -0.5), at(0.38, 0.0), at(0.0, 0.5), at(-0.38, 0.0)], sh, a, false);
            }
            "wave" => {
                let ls: Vec<Vec<P>> = (0..3).map(|k| (0..=12).map(|i| { let t = i as f64 / 12.0; at(-1.0 + t * 2.0, -0.6 + k as f64 * 0.6 + (t * 9.0 + k as f64).sin() * 0.18) }).collect()).collect();
                stroke_lines(ctx, &ls, col, a, 0.45);
            }
            "flame" => {
                for k in 0..3 {
                    let o = (k as f64 - 1.0) * 0.6;
                    let hgt = if k == 1 { 1.1 } else { 0.8 };
                    fill_poly(ctx, &[at(o - 0.3, 0.9), at(o - 0.38, 0.1), at(o, -hgt), at(o + 0.15, -0.2), at(o + 0.38, 0.1), at(o + 0.3, 0.9)], col, a, true);
                }
            }
            "yin" => {
                ctx.begin(); ctx.move_to(p.x + r, p.y); ctx.arc(p.x, p.y, r, 0.0, PI * 2.0); ctx.stroke_w(&sol(col, a), 0.4);
                let half: Vec<P> = (0..=12).map(|i| { let t = -PI / 2.0 + i as f64 / 12.0 * PI; at(t.cos(), t.sin()) }).collect();
                let s2: Vec<P> = (0..=8).map(|i| { let t = PI / 2.0 + i as f64 / 8.0 * PI; at(t.cos() * 0.5, 0.5 + t.sin() * 0.5) }).collect();
                let mut pts = half; pts.extend(s2);
                fill_poly(ctx, &pts, col, a, false);
            }
            _ => fill_poly(ctx, &star_pts(p, r, rot), col, a, false),
        }
    }

    // ------------------------------------------------------------------ belt
    pub(super) fn belt_detail(&self, ctx: &mut Ctx) {
        let (sk, c, style) = (&self.sk, &self.c, self.style);
        let (w, s) = (sk.w, sk.s);
        let tl = sk.l.torso / 29.0;
        let Some(bv) = gv(style, "belt") else { return };
        let bc = if bv == &Value::Bool(true) { c.c("belt") } else { c.cref(Some(bv), "") };
        let t = |v: f64, u: f64| self.t(v * w, u * tl);
        if let Some(sc) = self.stitch_col(bc) {
            self.seam(ctx, &[t(-8.0, 1.3), t(0.0, 1.5), t(8.2, 1.3)], sc, 0.8);
            self.seam(ctx, &[t(-7.6, 4.2), t(0.0, 4.4), t(7.8, 4.2)], sc, 0.8);
        }
        let ac = c.c("accent");
        let kind = self.ss("buckle").unwrap_or("");
        let p = t(4.3, 2.75);
        let r = 2.1 * s;
        let fr = ang(sk.fwd);
        match kind {
            "round" => { fill_ell(ctx, p, r, r, 0.0, mix(ac, [0, 0, 0], 0.5), 1.0); fill_ell(ctx, p, r * 0.9, r * 0.9, 0.0, ac, 1.0); fill_ell(ctx, p, r * 0.5, r * 0.5, 0.0, shade_of(bc), 1.0); fill_ell(ctx, add(p, light(), 0.35 * s), r * 0.28, r * 0.2, 0.0, WHITE, 0.55); }
            "square" => {
                let sq = |k: f64| [add(add(p, sk.fwd, -r * k), sk.up, -r * k), add(add(p, sk.fwd, r * k), sk.up, -r * k), add(add(p, sk.fwd, r * k), sk.up, r * k), add(add(p, sk.fwd, -r * k), sk.up, r * k)];
                fill_poly(ctx, &sq(1.05), mix(ac, [0, 0, 0], 0.5), 1.0, false);
                fill_poly(ctx, &sq(0.92), ac, 1.0, false);
                fill_poly(ctx, &sq(0.5), shade_of(bc), 1.0, false);
                stroke_lines(ctx, &[vec![add(add(p, sk.fwd, -r * 0.9), sk.up, r * 0.9), add(add(p, sk.fwd, r * 0.9), sk.up, r * 0.9)]], WHITE, 0.5, 0.2);
            }
            "diamond" | "star" => {
                let pts: Vec<P> = if kind == "star" { star_pts(p, r * 1.3, 0.0) } else { vec![add(p, sk.up, r * 1.2), add(p, sk.fwd, r * 1.2), add(p, sk.up, -r * 1.2), add(p, sk.fwd, -r * 1.2)] };
                fill_poly(ctx, &pts, mix(ac, [0, 0, 0], 0.45), 1.0, false);
                let inner: Vec<P> = pts.iter().map(|q| lerp(*q, p, 0.22)).collect();
                fill_poly(ctx, &inner, ac, 1.0, false);
                fill_ell(ctx, p, r * 0.3, r * 0.3, 0.0, self.c.c("energy"), 0.95);
            }
            "skull" => {
                fill_ell(ctx, p, r * 1.05, r * 1.15, fr, mix(ac, [0, 0, 0], 0.5), 1.0);
                fill_ell(ctx, p, r * 0.95, r * 1.05, fr, [0xea, 0xe4, 0xd4], 1.0);
                fill_ell(ctx, add(add(p, sk.fwd, -r * 0.3), sk.up, r * 0.15), r * 0.26, r * 0.3, 0.0, [0x20, 0x18, 0x20], 1.0);
                fill_ell(ctx, add(add(p, sk.fwd, r * 0.3), sk.up, r * 0.15), r * 0.26, r * 0.3, 0.0, [0x20, 0x18, 0x20], 1.0);
                stroke_lines(ctx, &[vec![add(p, sk.up, -r * 0.55), add(p, sk.up, -r * 0.95)]], [0x20, 0x18, 0x20], 0.9, 0.22);
            }
            _ => {}
        }
        // pouches + talisman hanging from the belt
        let np = self.sn("pouches", 0.0) as i32;
        for i in 0..np {
            let at0 = t(-4.5 - i as f64 * 3.0, 1.5);
            let dn = norm(add(pt(0.0, 1.0), dir(self.trail), 0.15));
            let a = add(at0, dn, 0.4);
            let (pw, ph) = (2.6 * s, 3.8 * s);
            let pc = self.c.get("pouch").unwrap_or(mix(bc, [0x8a, 0x66, 0x44], 0.5));
            let cn = [add(a, perp(dn), -pw), add(a, perp(dn), pw), add(add(a, perp(dn), pw * 1.1), dn, ph), add(add(a, perp(dn), -pw * 1.1), dn, ph)];
            fill_poly(ctx, &cn, mix(pc, [0, 0, 0], 0.5), 1.0, false);
            let inner: Vec<P> = cn.iter().map(|p| lerp(*p, lerp(cn[0], cn[2], 0.5), 0.08)).collect();
            fill_poly(ctx, &inner, pc, 1.0, false);
            fill_poly(ctx, &[cn[0], cn[1], lerp(cn[1], cn[2], 0.38), lerp(cn[0], cn[3], 0.38)], light_of(pc), 0.55, false);
            stroke_lines(ctx, &[vec![lerp(cn[0], cn[3], 0.38), lerp(cn[1], cn[2], 0.38)]], shade_of(pc), 0.9, 0.3);
            dots(ctx, &[lerp(lerp(cn[0], cn[3], 0.38), lerp(cn[1], cn[2], 0.38), 0.5)], 0.32 * s, ac, 1.0);
            self.seam(ctx, &[lerp(cn[0], cn[3], 0.75), lerp(cn[1], cn[2], 0.75)], light_of(pc), 0.7);
        }
        if let Some(tg) = self.sv("talisman") {
            let col = self.col_of(gv(tg, "color"), c.c("cloth"));
            let a = t(-1.0, 1.0);
            let dn = norm(add(pt(0.0, 1.0), dir(self.trail), 0.25));
            let b = add(a, dn, 6.0 * s);
            stroke_lines(ctx, &[vec![a, b]], mix(col, [0, 0, 0], 0.4), 1.0, 0.35);
            let cn = [add(b, perp(dn), -1.6 * s), add(b, perp(dn), 1.6 * s), add(add(b, perp(dn), 1.6 * s), dn, 4.2 * s), add(add(b, perp(dn), -1.6 * s), dn, 4.2 * s)];
            fill_poly(ctx, &cn, mix(col, [0xff, 0xf0, 0xd0], 0.1), 1.0, false);
            stroke_poly(ctx, &cn, [0x20, 0x18, 0x20], 0.9, 0.25, false);
            stroke_lines(ctx, &[vec![lerp(cn[0], cn[3], 0.3), lerp(cn[1], cn[2], 0.3)], vec![lerp(cn[0], cn[3], 0.55), lerp(cn[1], cn[2], 0.55)], vec![lerp(lerp(cn[0], cn[1], 0.5), lerp(cn[3], cn[2], 0.5), 0.1), lerp(lerp(cn[0], cn[1], 0.5), lerp(cn[3], cn[2], 0.5), 0.85)]], [0x20, 0x18, 0x20], 0.8, 0.2);
            let tassel: Vec<Vec<P>> = (0..3).map(|i| { let o = (i as f64 - 1.0) * 0.7 * s; let st = add(lerp(cn[3], cn[2], 0.5), perp(dn), o); vec![st, add(add(st, dn, 2.6 * s), perp(dn), o * 0.5 + (self.phase + i as f64).sin() * 0.3)] }).collect();
            stroke_lines(ctx, &tassel, col, 1.0, 0.4);
        }
    }

    // ------------------------------------------------------------------ cloth panels
    /// robe / skirt / coat tail: drape folds, hem band, stitching, weave
    pub(super) fn flap_detail(&self, ctx: &mut Ctx, pts: &[P], col: Rgb, front: bool) {
        if pts.len() < 8 { return; }
        let al = if front { 1.0 } else { 0.75 };
        let (folds, weave) = (self.sn("folds", 0.0).min(1.0), self.ss("weave").unwrap_or(""));
        let stitch = self.stitch_col(col);
        ctx.save();
        clip_geo(ctx, &[Geo::Poly(pts.to_vec(), true)]);
        if !weave.is_empty() && front {
            let u = norm(sub(lerp(pts[2], pts[5], 0.5), lerp(pts[0], pts[7], 0.5)));
            self.weave(ctx, weave, col, &[Geo::Poly(pts.to_vec(), true)], lerp(pts[0], pts[7], 0.5), u, al);
        }
        let (l0, l1) = (lerp(pts[0], pts[7], 0.5), lerp(pts[1], pts[6], 0.5));
        let l2 = lerp(pts[2], pts[5], 0.5);
        if folds > 0.0 {
            let n = 4;
            let mut dk = vec![];
            let mut lt = vec![];
            for k in 0..n {
                let f = (k as f64 + 0.5) / n as f64;
                let a = lerp(pts[0], pts[7], f);
                let b = lerp(pts[1], pts[6], f + (hash(k as f64 + 2.0) - 0.5) * 0.15);
                let e = lerp(pts[2], pts[5], f + (hash(k as f64 + 6.0) - 0.5) * 0.2);
                let pl = qc(lerp(a, b, 0.25), b, e, 8);
                lt.push(pl.iter().map(|p| add(*p, light(), 0.5)).collect::<Vec<P>>());
                dk.push(pl);
            }
            stroke_lines(ctx, &dk, shade_of(col), 0.55 * folds * al, 0.55);
            stroke_hi(ctx, &lt, light_of(col), 0.24 * folds * al, 0.34);
            // shade the trailing edge, light the leading one
            fill_poly(ctx, &[pts[7], pts[6], pts[5], lerp(pts[6], pts[5], 0.5), lerp(pts[7], pts[6], 0.3)], shade_of(col), 0.18 * folds * al, true);
        }
        // hem band
        let hem_c = self.c.get("trim").unwrap_or(shade_of(col));
        if self.sv("hem").is_some() {
            let hc = self.col_of(self.sv("hem"), hem_c);
            let a = lerp(pts[1], pts[6], 0.0);
            let _ = (a, l0, l1);
            let e0 = lerp(l1, l2, 0.78);
            fill_poly(ctx, &[lerp(pts[1], pts[2], 0.78), lerp(pts[6], pts[5], 0.78), pts[5], pts[4], pts[3], pts[2]], hc, 0.95 * al, false);
            let _ = e0;
            stroke_lines(ctx, &[vec![lerp(pts[1], pts[2], 0.78), lerp(pts[6], pts[5], 0.78)]], mix(hc, [0, 0, 0], 0.4), 0.8 * al, 0.3);
            stroke_hi(ctx, &[vec![add(lerp(pts[1], pts[2], 0.84), light(), 0.0), add(lerp(pts[6], pts[5], 0.84), light(), 0.0)]], light_of(hc), 0.4 * al, 0.22);
        }
        if let Some(sc) = stitch {
            self.seam(ctx, &[lerp(pts[1], pts[2], 0.7), lerp(pts[6], pts[5], 0.7)], sc, 0.75 * al);
        }
        ctx.restore();
    }

    pub(super) fn cape_detail(&self, ctx: &mut Ctx, a: &[P], b: &[P], col: Rgb) {
        let (folds, weave) = (self.sn("folds", 0.0).min(1.0), self.ss("weave").unwrap_or(""));
        let n = a.len().min(b.len());
        if n < 3 || (folds <= 0.0 && weave.is_empty() && self.stitch_col(col).is_none()) { return; }
        let mut pts = a.to_vec();
        let mut bb = b.to_vec();
        bb.reverse();
        pts.extend(bb);
        let cape_pts = pts.clone();
        ctx.save();
        clip_geo(ctx, &[Geo::Poly(pts, true)]);
        if folds > 0.0 {
            let mut dk = vec![];
            let mut lt = vec![];
            for k in 0..6 {
                let f = (k as f64 + 0.5) / 6.0;
                let pl: Vec<P> = (0..n).map(|i| { let wob = (i as f64 * 0.9 + k as f64 * 1.7 + self.phase).sin() * 0.7; add(lerp(a[i], b[i], f + wob * 0.03), pt(0.0, 1.0), wob * 0.3) }).collect();
                lt.push(pl.iter().map(|p| add(*p, light(), 0.6)).collect::<Vec<P>>());
                dk.push(pl);
            }
            stroke_lines(ctx, &dk, shade_of(col), 0.45 * folds, 0.6);
            stroke_hi(ctx, &lt, light_of(col), 0.2 * folds, 0.35);
        }
        if !weave.is_empty() {
            let u = norm(sub(a[n - 1], a[0]));
            self.weave(ctx, weave, col, &[Geo::Poly(cape_pts.clone(), true)], a[0], u, 0.9);
        }
        if let Some(sc) = self.stitch_col(col) {
            let e: Vec<P> = (1..n).map(|i| lerp(a[i], b[i], 0.07)).collect();
            self.seam(ctx, &e, sc, 0.7);
        }
        ctx.restore();
    }

    // ------------------------------------------------------------------ pauldrons
    pub(super) fn pauldron_shapes(&self, front: bool) -> Vec<Shape> {
        let sk = &self.sk;
        let w = sk.w;
        let (sh, el) = if front { (sk.sh_f, sk.el_f) } else { (sk.sh_b, sk.el_b) };
        let d = if front { dir(sk.up_af) } else { norm(sub(el, sh)) };
        let rot = d.y.atan2(d.x) + PI / 2.0;
        let kind = self.ss("pauldron").unwrap_or("round");
        let m = self.c.c("metal");
        let dk = |c: Rgb| if front { c } else { back_of(c) };
        let k = if front { 1.0 } else { 0.85 };
        let n = perp(d);
        let c0 = add(sh, d, 1.5);
        let mut out = vec![];
        match kind {
            "layered" => {
                for i in (0..3).rev() {
                    let fi = i as f64;
                    out.push(shp(Geo::Ellipse(add(sh, d, 0.5 + fi * 2.6 * w), (5.0 + fi * 0.9) * w * k, (3.6 + fi * 0.5) * w * k, rot), dk(mix(m, [0, 0, 0], 0.08 * (2.0 - fi))), 1.6).rim(0.6).line(true));
                }
            }
            "spiked" => {
                for i in 0..3 {
                    let t = -0.55 + i as f64 * 0.55;
                    let base = add(add(c0, n, t * 6.0 * w), d, -2.4 * w);
                    let tip = add(add(base, n, t * 4.0 * w), d, -5.6 * w * (1.0 - t.abs() * 0.3));
                    out.push(shp(poly(vec![add(base, n, -1.8 * w), tip, add(base, n, 1.8 * w)], false), dk(light_of(m)), 1.0).line(true));
                }
                out.push(shp(Geo::Ellipse(c0, 6.4 * w * k, 4.6 * w * k, rot), dk(m), 1.8).rim(0.6).line(true));
                out.push(shp(Geo::Ellipse(add(c0, d, 0.4), 4.0 * w * k, 2.6 * w * k, rot), dk(shade_of(m)), 0.0).line(true));
            }
            "fur" => {
                let nn = 14;
                let pts: Vec<P> = (0..nn).map(|i| { let a = i as f64 / nn as f64 * PI * 2.0; let r = if i % 2 == 0 { 1.0 } else { 1.28 + hash(i as f64) * 0.2 }; add(add(c0, n, a.cos() * 6.4 * w * r * k), d, a.sin() * 4.6 * w * r * k) }).collect();
                out.push(shp(poly(pts, false), dk(self.c.get("fur").unwrap_or([0xe8, 0xe0, 0xd0])), 2.4).rim(0.9).line(true));
            }
            "horned" => {
                let base = add(c0, n, -2.0 * w);
                let sp: Vec<P> = (0..=6).map(|i| { let t = i as f64 / 6.0; add(add(base, d, -t * 9.0 * w), n, -t * t * 6.0 * w - t * 2.0) }).collect();
                out.push(shp(poly(ribbon_pts(&sp, 4.6 * w, 0.4), true), dk(self.c.c("horn")), 1.4).line(true));
                out.push(shp(Geo::Ellipse(c0, 6.2 * w * k, 4.4 * w * k, rot), dk(m), 1.8).rim(0.6).line(true));
            }
            _ => out.push(shp(Geo::Ellipse(c0, 6.2 * w * k, 4.4 * w * k, rot), dk(m), 1.8).rim(0.6).line(true)),
        }
        // bevelled inner plate
        if self.sv("armor").is_some() && kind == "round" {
            let mut inner = shp(Geo::Ellipse(add(c0, d, 0.5), 4.4 * w * k, 2.9 * w * k, rot), dk(shade_of(m)), 1.0);
            inner.no_outline = true;
            out.push(inner);
            let mut hi = shp(Geo::Ellipse(add(add(c0, d, 0.1), light(), -0.3), 3.8 * w * k, 2.4 * w * k, rot), dk(light_of(m)), 0.0);
            hi.no_outline = true;
            hi.flat = true;
            out.push(hi);
            if self.sv("armor").is_some_and(|a| gt(a, "rivets")) {
                for t in [-1.0, 0.0, 1.0] {
                    let mut r = shp(Geo::Ellipse(add(add(c0, n, t * 4.4 * w * k), d, 1.4 - t.abs() * 0.9), 0.5, 0.5, 0.0), dk(light_of(m)), 0.0);
                    r.no_outline = true;
                    r.flat = true;
                    out.push(r);
                }
            }
        }
        out
    }
}
