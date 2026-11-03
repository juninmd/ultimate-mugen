//! Per-stage animated content: neon city, bamboo grove, plains, spirit gate, volcano. (Dojo lives in the parent module.)
use super::*;

fn vis(p: Vec2, pad: f32) -> bool { p.x > -pad && p.x < screen_width() + pad }

impl StageReact {
    /// A swaying bamboo stalk in the local coordinates of layer `li`: tapered strip with moon-lit edge, node rings and drooping leaves.
    #[allow(clippy::too_many_arguments)]
    fn stalk(&self, cx: &Ctx, li: usize, x: f32, base: f32, h: f32, w: f32, ph: f32, lit: Color, shade: Color, fog: Color, fog_a: f32, leaves: bool) {
        let s = cx.s;
        if !vis(cx.l(li, x, base), 70.0 * s / 3.0 + 40.0) { return; }
        let wx = cx.lw(li, x);
        let sway = self.wind(wx, ph) * 7.0 + self.push_at(wx) * 15.0;
        let n = ((h / 24.0) as usize).max(4);
        let (mut a, mut c, mut b, mut cl, mut cm, mut cr) = (vec![], vec![], vec![], vec![], vec![], vec![]);
        let mut nodes = vec![];
        for k in 0..=n {
            let f = k as f32 / n as f32;
            let bx = sway * f.powf(1.7) + (self.t * 1.5 + ph + f * 2.0).sin() * 0.8 * f;
            let p = cx.l(li, x + bx, base - h * f);
            let hw = w * 0.5 * (1.0 - 0.35 * f) * s;
            a.push(p - vec2(hw, 0.0)); c.push(p + vec2(-hw * 0.15, 0.0)); b.push(p + vec2(hw, 0.0));
            let fa = fog_a * (1.0 - f).powf(1.2);
            cl.push(lerpc(lit, fog, fa)); cm.push(lerpc(lerpc(lit, shade, 0.5), fog, fa)); cr.push(lerpc(shade, fog, fa));
            nodes.push(p);
        }
        strip(&a, &c, &cl, &cm); strip(&c, &b, &cm, &cr);
        for (k, p) in nodes.iter().enumerate().skip(1) {
            let hw = w * 0.5 * (1.0 - 0.35 * k as f32 / n as f32) * s;
            let dark = ca(shade, 0.9);
            seg(*p - vec2(hw * 1.25, 0.0), *p + vec2(hw * 1.25, 0.0), 0.9 * s * (w / 5.0).max(0.6), dark, dark);
        }
        if leaves {
            for k in (n * 5 / 8)..=n {
                for side in [-1.0f32, 1.0] {
                    if hash(ph as u32 * 31 + k as u32, side as u32 + 5) < 0.25 { continue; }
                    let lf = (self.t * 2.4 + ph * 3.0 + k as f32 * 1.7 + side).sin() * 0.18 + self.wind(wx, ph + k as f32) * 0.3;
                    let ang = side * (0.45 + hash(k as u32, 9) * 0.5) + lf * side * 0.6 + 0.5;
                    let l = (8.0 + hash(k as u32 + ph as u32, 3) * 7.0) * s * (w / 5.0).clamp(0.8, 1.6);
                    let p = nodes[k];
                    let d = rot(vec2(side * l, 0.0), side * ang * 0.8); let nrm = vec2(-d.y, d.x).normalize_or_zero() * l * 0.16;
                    let tip = p + d;
                    tr([p, p + d * 0.5 + nrm, tip], [lit, lerpc(lit, shade, 0.4), shade]); tr([p, p + d * 0.5 - nrm, tip], [lit, lerpc(lit, shade, 0.7), shade]);
                }
            }
        }
    }

    // ───────────────────────────────────────── NEON ─────────────────────────────────────────
    pub(super) fn neon_layer(&self, i: usize, cx: &Ctx, g: &Gfx) {
        let (s, t) = (cx.s, self.t);
        match i {
            0 => additive(g, || {
                for k in 0..3 {
                    let base = cx.l(0, 70.0 + k as f32 * 190.0, 222.0);
                    let a = -1.57 + (t * 0.33 + k as f32 * 2.1).sin() * 0.55; let d = vec2(a.cos(), a.sin()); let nv = vec2(-d.y, d.x);
                    let tip = base + d * 250.0 * s; let col = [hx(0xb8f0ff, 1.0), hx(0xffb8f0, 1.0), hx(0xd8c8ff, 1.0)][k];
                    qd([base - nv * 1.5 * s, base + nv * 1.5 * s, tip + nv * 14.0 * s, tip - nv * 14.0 * s], [ca(col, 0.16), ca(col, 0.16), ca(col, 0.0), ca(col, 0.0)]);
                }
                for k in 0..4u32 { let x = (t * (6.0 + k as f32 * 2.0) + k as f32 * 160.0) % 600.0 - 40.0; let y = 60.0 + k as f32 * 22.0 + (t + k as f32).sin() * 3.0;
                    let on = (t * 2.0 + k as f32).sin() > -0.3; if on { fan(cx.l(0, x, y), 4.0 * s, 4.0 * s, 8, hx(if k % 2 == 0 { 0xff4040 } else { 0xffffff }, 0.8), hx(0xff4040, 0.0)); } }
            }),
            1 => {
                // blinking aircraft lights on the tallest towers
                additive(g, || {
                    for (n, (x, w, h)) in skyline(21, 760.0, 16.0, 34.0, 60.0, 170.0, 8.0).into_iter().enumerate() {
                        if h > 120.0 && (t * 1.6 + n as f32 * 0.7).sin() > 0.4 { fan(cx.l(1, x + w / 2.0, 200.0 - h - 3.0), 5.0 * s, 5.0 * s, 8, hx(0xff3030, 0.9), hx(0xff3030, 0.0)); }
                    }
                    // flying cars with light trails
                    for k in 0..3u32 { let sp = 40.0 + k as f32 * 22.0; let x = (t * sp + k as f32 * 260.0) % 900.0 - 80.0; let y = 54.0 + k as f32 * 34.0;
                        let (h_, t_) = (cx.l(1, x, y), cx.l(1, x - 20.0, y)); seg(h_, t_, 1.2 * s, hx(0xffe8c0, 0.9), hx(0xff6ad0, 0.0)); fan(h_, 3.0 * s, 3.0 * s, 8, hx(0xffffff, 0.9), hx(0xffa0e0, 0.0)); }
                });
                // advertising blimp with a scrolling LED screen
                let bx = (t * 5.0) % 1000.0 - 120.0; let c = cx.l(1, bx, 86.0);
                fan(c, 38.0 * s, 10.0 * s, 20, hx(0x2c1a58, 1.0), hx(0x140a30, 1.0));
                let hue = |q: f32| hx(NEON_RGB[((t * 0.8 + q) as usize) % 5], 1.0);
                for q in 0..6 { let u = -26.0 + q as f32 * 9.0; qd([c + vec2(u * s, -3.0 * s), c + vec2((u + 7.0) * s, -3.0 * s), c + vec2((u + 7.0) * s, 3.0 * s), c + vec2(u * s, 3.0 * s)], [hue(q as f32); 4]); }
                additive(g, || fan(c, 52.0 * s, 20.0 * s, 16, hx(0xff6ad0, 0.12), hx(0xff6ad0, 0.0)));
            }
            2 => additive(g, || {
                for (n, (x, y, w, h, c)) in neon_signs().into_iter().enumerate() {
                    let ctr = cx.l(2, x + w / 2.0, y + h / 2.0);
                    if !vis(ctr, 120.0) { continue; }
                    let fl = self.flick(n as u32);
                    let col = hx(NEON_RGB[c], 1.0);
                    fan(ctr, (w.max(h) * 0.75) * s, (w.max(h) * 0.75) * s, 12, ca(col, 0.26 * fl), ca(col, 0.0));
                    let (p0, p1, p2, p3) = (cx.l(2, x, y), cx.l(2, x + w, y), cx.l(2, x + w, y + h), cx.l(2, x, y + h));
                    let o = ca(col, 0.85 * fl.min(1.3));
                    for (a, b) in [(p0, p1), (p1, p2), (p2, p3), (p3, p0)] { seg(a, b, 1.6 * s, o, o); }
                    for q in 0..3 { let yy = y + 5.0 + q as f32 * h / 3.4; seg(cx.l(2, x + 2.5, yy), cx.l(2, x + w - 2.5, yy + 2.0), 1.3 * s, ca(col, 0.8 * fl), ca(col, 0.8 * fl)); }
                }
                // holographic billboard with scan lines; glitches when the stage is shaken
                let glitch = self.flicker > 0.35 && ((t * 24.0) as u32 % 3 == 0);
                let gx = if glitch { (hash((t * 24.0) as u32, 3) - 0.5) * 8.0 } else { 0.0 };
                for q in 0..8u32 {
                    let y = 58.0 + q as f32 * 5.0; let col = hx(NEON_RGB[((q + (t * 1.5) as u32) % 5) as usize], 1.0);
                    let len = 40.0 + 20.0 * ((t * 1.3 + q as f32).sin() * 0.5 + 0.5);
                    qd([cx.l(2, 470.0 + gx, y), cx.l(2, 470.0 + len + gx, y), cx.l(2, 470.0 + len + gx, y + 3.5), cx.l(2, 470.0 + gx, y + 3.5)], [ca(col, 0.32), ca(col, 0.12), ca(col, 0.12), ca(col, 0.32)]);
                }
                fan(cx.l(2, 500.0, 78.0), 42.0 * s, 28.0 * s, 12, hx(0x60e8ff, 0.12 * self.flick(41)), hx(0x60e8ff, 0.0));
            }),
            3 => {
                // fans behind the barrier waving glow sticks
                for k in 0..24u32 {
                    let x = 24.0 + k as f32 * 46.0 + hash(k, 1) * 14.0; let ph = hash(k, 2) * 6.28;
                    let jump = self.cheer * (0.5 + 0.5 * (t * 10.0 + ph).sin().abs()) * 8.0 + (t * 2.0 + ph).sin().max(0.0) * 0.5;
                    let body = lerpc(hx(0x120a26, 1.0), hx(0x2a1a50, 1.0), hash(k, 3));
                    let st = hx(NEON_RGB[(hash(k, 4) * 5.0) as usize % 5], 1.0);
                    spectator(cx.l(3, x, 208.0), s, 32.0, jump, (self.cheer * 1.5 + 0.18).min(1.0), body, hx(0x0e0820, 1.0), lerpc(body, st, 0.18), Some(st), t, ph);
                }
                additive(g, || { for k in 0..24u32 { let st = hx(NEON_RGB[(hash(k, 4) * 5.0) as usize % 5], 1.0); let x = 24.0 + k as f32 * 46.0 + hash(k, 1) * 14.0; fan(cx.l(3, x, 178.0 - self.cheer * 12.0), 6.0 * s, 6.0 * s, 8, ca(st, 0.5), ca(st, 0.0)); } });
            }
            4 => additive(g, || {
                // neon signs reflected in the wet street
                let top = cx.w(0.0, 0.0).y;
                for (n, (x, _y, w, _h, c)) in neon_signs().into_iter().enumerate() {
                    let fl = self.flick(n as u32); let col = hx(NEON_RGB[c], 1.0);
                    let sx = cx.l(2, x + w / 2.0, 0.0).x;
                    if !vis(vec2(sx, 0.0), 40.0) { continue; }
                    for k in 0..7u32 {
                        let y0 = top + k as f32 * 5.0 * s; let wob = (t * 2.5 + n as f32 + k as f32 * 0.9).sin() * 2.2 * s;
                        let wd = (w * 0.5 + 3.0) * s * (1.0 - k as f32 * 0.07);
                        qd([vec2(sx - wd + wob, y0), vec2(sx + wd + wob, y0), vec2(sx + wd + wob, y0 + 5.0 * s), vec2(sx - wd + wob, y0 + 5.0 * s)], [ca(col, 0.13 * fl * (1.0 - k as f32 / 8.0)); 4]);
                    }
                }
            }),
            _ => {}
        }
    }

    pub(super) fn neon_front(&self, cx: &Ctx, g: &Gfx) {
        let (s, t) = (cx.s, self.t);
        // rising steam from street vents (lit magenta from the signs)
        for (v, vx) in [300.0f32, 780.0].iter().enumerate() {
            for k in 0..6 {
                let p = (t * (0.28 + self.gust * 0.05) + k as f32 / 6.0 + v as f32 * 0.37) % 1.0;
                let c = cx.l(4, *vx + (p * 7.0 + k as f32).sin() * 5.0 * p + p * 14.0 * self.wind(*vx, 2.0), 64.0 - p * 74.0);
                let r = (4.0 + p * 15.0) * s;
                fan(c, r, r * 0.9, 12, hx(0xc8a0ff, 0.20 * (1.0 - p) * (1.0 - p) + 0.02), hx(0xc8a0ff, 0.0));
            }
        }
        // sagging power cables that sway and spark
        for (ci, (y0, y1, sag)) in [(24.0f32, 34.0f32, 16.0f32), (40.0, 30.0, 11.0)].iter().enumerate() {
            let mut prev: Option<Vec2> = None;
            for k in 0..=14 {
                let f = k as f32 / 14.0;
                let y = y0 + (y1 - y0) * f + sag * 4.0 * f * (1.0 - f) + (t * 1.1 + f * 3.0 + ci as f32).sin() * 0.9 + self.swing * 7.0 * (f * 3.14).sin() + self.shake * (t * 30.0 + f * 9.0).sin() * 1.1;
                let p = cx.l(5, -10.0 + f * 580.0, y);
                if let Some(q) = prev { seg(q, p, 1.1 * s, hx(0x0a0612, 1.0), hx(0x0a0612, 1.0)); }
                prev = Some(p);
            }
        }
        // street lamps at the screen edges
        for (n, x) in [46.0f32, 514.0].iter().enumerate() {
            let sgn = if n == 0 { 1.0 } else { -1.0 };
            let (b, top, head) = (cx.l(5, *x, 256.0), cx.l(5, *x, 88.0), cx.l(5, *x + sgn * 16.0, 86.0));
            seg(b, top, 3.0 * s, hx(0x120a22, 1.0), hx(0x1c1234, 1.0)); seg(top, head, 2.0 * s, hx(0x1c1234, 1.0), hx(0x1c1234, 1.0));
            let fl = self.flick(500 + n as u32);
            qd([head + vec2(-5.0 * s, 0.0), head + vec2(5.0 * s, 0.0), head + vec2(3.5 * s, 3.0 * s), head + vec2(-3.5 * s, 3.0 * s)], [hx(0x2c1c4a, 1.0), hx(0x2c1c4a, 1.0), hx(0xfff0c0, fl.min(1.0)), hx(0xfff0c0, fl.min(1.0))]);
            additive(g, || {
                qd([head + vec2(-3.0 * s, 3.0 * s), head + vec2(3.0 * s, 3.0 * s), head + vec2(sgn * 30.0 * s + 40.0 * s, 150.0 * s), head + vec2(sgn * 30.0 * s - 40.0 * s, 150.0 * s)], [hx(0xffe0a0, 0.13 * fl), hx(0xffe0a0, 0.13 * fl), hx(0xffe0a0, 0.0), hx(0xffe0a0, 0.0)]);
                fan(head + vec2(0.0, 3.0 * s), 22.0 * s, 22.0 * s, 14, hx(0xffe0a0, 0.4 * fl), hx(0xffe0a0, 0.0));
            });
        }
        // rain-drop ripples are spawned in begin_frame; flicker overlay on big hits
        if self.flicker > 0.5 { additive(g, || { let (w, h) = cx.view(); qd([cx.v(0.0, 0.0), cx.v(w, 0.0), cx.v(w, h), cx.v(0.0, h)], [hx(0xb090ff, 0.05 * self.flicker); 4]); }); }
    }

    // ───────────────────────────────────────── BAMBOO ─────────────────────────────────────────
    pub(super) fn bamboo_layer(&self, i: usize, cx: &Ctx, g: &Gfx) {
        let (s, t) = (cx.s, self.t);
        match i {
            0 => additive(g, || {
                let m = cx.l(0, 182.0, 55.0);
                let br = 1.0 + 0.08 * (t * 0.7).sin() + self.flicker * 0.15;
                fan(m, 70.0 * s * br, 70.0 * s * br, 24, hx(0xc8fff4, 0.20), hx(0xc8fff4, 0.0));
                for k in 0..26u32 { let tw = (t * (1.0 + hash(k, 3) * 2.0) + k as f32 * 2.3).sin(); if tw > 0.5 { fan(cx.l(0, hash(k, 1) * 520.0, hash(k, 2) * 120.0), 2.4 * s, 2.4 * s, 6, hx(0xe0ffff, 0.8 * (tw - 0.5) * 2.0), hx(0xe0ffff, 0.0)); } }
            }),
            1 => {
                // moon shafts + drifting ground fog
                additive(g, || { for k in 0..3u32 {
                    let x = 90.0 + k as f32 * 270.0 + (t * 0.15 + k as f32).sin() * 12.0; let a = 0.30 + (t * 0.3 + k as f32 * 1.7).sin() * 0.04;
                    qd([cx.l(1, x, -10.0), cx.l(1, x + 26.0, -10.0), cx.l(1, x + 26.0 - 60.0 * a * 3.0, 230.0), cx.l(1, x - 40.0 * a * 3.0, 230.0)], [hx(0xc8fff0, 0.08), hx(0xc8fff0, 0.08), hx(0xc8fff0, 0.0), hx(0xc8fff0, 0.0)]); } });
                for k in 0..4u32 { let x = (hash(k, 1) * 1000.0 + t * (4.0 + hash(k, 2) * 4.0)) % 1100.0 - 100.0; fan(cx.l(1, x, 205.0 - hash(k, 3) * 20.0), 140.0 * s, 20.0 * s, 16, hx(0x9fd6d0, 0.13), hx(0x9fd6d0, 0.0)); }
            }
            2 => {
                for k in 0..26u32 {
                    let x = hash(k, 11) * 900.0; let ph = hash(k, 12) * 6.28;
                    self.stalk(cx, 2, x, 252.0, 232.0 + hash(k, 13) * 28.0, 3.4 + hash(k, 14) * 1.6, ph, hx(0x2c7a70, 1.0), hx(0x0e3636, 1.0), hx(0x3a8a86, 1.0), 0.65, true);
                }
            }
            3 => {
                for k in 0..11u32 {
                    let x = 30.0 + k as f32 * 98.0 + hash(k, 21) * 40.0; let ph = hash(k, 22) * 6.28;
                    self.stalk(cx, 3, x, 252.0, 250.0, 7.0 + hash(k, 23) * 2.5, ph, hx(0x2a6e64, 1.0), hx(0x051e1e, 1.0), hx(0x2a6a68, 1.0), 0.3, true);
                }
            }
            4 => {
                // moon glitter on the water
                additive(g, || {
                    let mx = cx.l(0, 182.0, 0.0).x;
                    for k in 0..16u32 { let y = cx.w(0.0, 2.0 + k as f32 * 2.1).y; let tw = ((t * 2.4 + k as f32 * 1.9).sin() * 0.5 + 0.5).powi(2); let wd = (4.0 + k as f32 * 1.6) * s * (0.5 + hash(k, 2));
                        let off = (t * 0.9 + k as f32).sin() * 5.0 * s; seg(vec2(mx - wd + off, y), vec2(mx + wd + off, y), 0.9 * s, ca(hx(0xe8fffa, 1.0), 0.5 * tw), ca(hx(0xe8fffa, 1.0), 0.5 * tw)); }
                });
            }
            _ => {}
        }
    }

    pub(super) fn bamboo_front(&self, cx: &Ctx, g: &Gfx) {
        let (s, t) = (cx.s, self.t);
        for (k, x) in [18.0f32, 548.0].iter().enumerate() {
            self.stalk(cx, 5, *x, 262.0, 292.0, 16.0, k as f32 * 2.0 + 0.5, hx(0x1a4a46, 1.0), hx(0x020c0c, 1.0), hx(0x103030, 1.0), 0.0, true);
        }
        // low mist crossing the pond
        for k in 0..4u32 { let x = (hash(k, 5) * 700.0 + t * (6.0 + hash(k, 6) * 6.0)) % 760.0 - 120.0; fan(cx.v(x, 228.0 - hash(k, 7) * 12.0), 130.0 * s, 16.0 * s, 16, hx(0xa8e0d8, 0.10), hx(0xa8e0d8, 0.0)); }
        // fireflies hovering over the water
        additive(g, || { for k in 0..16u32 {
            let x = (hash(k, 1) * 520.0 + (t * 0.3 + k as f32).sin() * 22.0) - cx.cam() * 0.2; let y = 120.0 + hash(k, 2) * 90.0 + (t * 0.5 + k as f32 * 1.7).cos() * 10.0;
            let b = (t * (1.2 + hash(k, 3)) + k as f32 * 2.3).sin() * 0.5 + 0.5;
            fan(cx.v(x.rem_euclid(560.0) - 60.0, y), 5.0 * s, 5.0 * s, 8, hx(0xd8ff70, 0.7 * b), hx(0xd8ff70, 0.0)); } });
    }

    // ───────────────────────────────────────── PLAINS ─────────────────────────────────────────
    pub(super) fn plains_layer(&self, i: usize, cx: &Ctx, g: &Gfx) {
        let (s, t) = (cx.s, self.t);
        match i {
            0 => {
                additive(g, || { let c = cx.l(0, 405.0, 40.0); for k in 0..7 { let a = 1.7 + k as f32 * 0.22 + (t * 0.05 + k as f32).sin() * 0.03;
                    tr([c, c + vec2(a.cos(), a.sin()) * 300.0 * s - vec2(14.0 * s, 0.0), c + vec2(a.cos(), a.sin()) * 300.0 * s + vec2(14.0 * s, 0.0)], [hx(0xfff4c0, 0.10), hx(0xfff4c0, 0.0), hx(0xfff4c0, 0.0)]); } });
                // drifting cumulus
                for k in 0..5u32 {
                    let sp = 2.0 + hash(k, 2) * 4.0 + self.gust * 3.0; let x = (hash(k, 1) * 700.0 + t * sp) % 700.0 - 100.0; let y = 36.0 + hash(k, 3) * 70.0; let sz = 0.8 + hash(k, 4) * 0.9;
                    for q in 0..6u32 { let (dx, dy, r) = ((q as f32 - 2.5) * 13.0 * sz, -(2.5 - (q as f32 - 2.5).abs()) * 3.5 * sz, (13.0 - (q as f32 - 2.5).abs() * 1.8) * sz);
                        fan(cx.l(0, x + dx, y + dy + 4.0), r * 1.1 * s, r * 0.8 * s, 12, hx(0xaab8d8, 0.35), hx(0xaab8d8, 0.0));
                        fan(cx.l(0, x + dx, y + dy), r * 1.1 * s, r * 0.9 * s, 12, hx(0xffffff, 0.55), hx(0xffffff, 0.0)); }
                }
            }
            1 => {
                for k in 0..2u32 {
                    let ang = t * 0.28 + k as f32 * 3.0; let c = cx.l(1, 260.0 + k as f32 * 280.0 + ang.cos() * 90.0, 78.0 + k as f32 * 20.0 + ang.sin() * 12.0);
                    let dir = if ang.sin() > 0.0 { -1.0 } else { 1.0 }; let flap = (t * 1.6 + k as f32).sin() * 0.25;
                    let col = hx(0x30201c, 0.95);
                    let (wl, wr) = (c + vec2(-10.0 * s, (-3.0 - flap * 6.0) * s), c + vec2(10.0 * s, (-3.0 - flap * 6.0) * s));
                    let mid = (c + wl) * 0.5 + vec2(0.0, -1.2 * s);
                    seg(wl, mid, 1.2 * s, col, col); seg(mid, c, 1.9 * s, col, col); seg(c, (c + wr) * 0.5 + vec2(0.0, -1.2 * s), 1.9 * s, col, col); seg((c + wr) * 0.5 + vec2(0.0, -1.2 * s), wr, 1.2 * s, col, col); let _ = dir;
                }
            }
            2 => { for k in 0..2u32 { let x = (hash(k, 1) * 1000.0 + t * (6.0 + self.gust * 6.0)) % 1160.0 - 80.0; fan(cx.l(2, x, 200.0 - hash(k, 2) * 12.0), 130.0 * s, 16.0 * s, 14, hx(0xf0d8b0, 0.07), hx(0xf0d8b0, 0.0)); } }
            3 => {
                // wind pump: lattice tower, whirling blades, tail vane — spins faster in gusts and shocks
                let base = cx.l(3, 530.0, 211.0); let top = cx.l(3, 530.0, 211.0 - 74.0);
                let leg = hx(0x3a2418, 1.0);
                for dx in [-13.0f32, 13.0] { seg(base + vec2(dx * s, 0.0), top + vec2(dx * 0.28 * s, 0.0), 1.4 * s, leg, leg); }
                for k in 0..5 { let f = k as f32 / 5.0; let y = f * 74.0; let wd = 13.0 * (1.0 - f * 0.72); seg(base + vec2(-wd * s, -y * s), base + vec2(wd * s, -(y + 14.0) * s), 0.7 * s, leg, leg); seg(base + vec2(wd * s, -y * s), base + vec2(-wd * s, -(y + 14.0) * s), 0.7 * s, leg, leg); }
                let hub = top + vec2(0.0, -3.0 * s);
                seg(hub, hub + vec2(34.0 * s, 1.0 * s), 1.0 * s, leg, leg);
                qd([hub + vec2(26.0 * s, -5.0 * s), hub + vec2(46.0 * s, -3.0 * s), hub + vec2(46.0 * s, 5.0 * s), hub + vec2(26.0 * s, 3.0 * s)], [hx(0xb88a60, 1.0), hx(0xa07850, 1.0), hx(0x7a5a40, 1.0), hx(0x8a6848, 1.0)]);
                for k in 0..18 { let a = self.rotor * 1.6 + k as f32 / 18.0 * 6.283; let (d0, d1) = (rot(vec2(6.0, 0.0), a), rot(vec2(26.0, 0.0), a)); let nrm = rot(vec2(0.0, 1.0), a) * 1.8 * s;
                    qd([hub + d0 * s - nrm, hub + d1 * s - nrm * 1.5, hub + d1 * s + nrm * 1.5, hub + d0 * s + nrm], [hx(0xc8a888, 1.0), hx(0xe8d0b0, 1.0), hx(0xe8d0b0, 1.0), hx(0xc8a888, 1.0)]); }
                crate::poly::circle(hub, 2.2 * s, hx(0x3a2418, 1.0));
                // tumbleweed bouncing across
                let x = (t * (28.0 + self.gust * 18.0) + 200.0) % 1300.0 - 100.0; let y = 211.0 - 7.0 - (t * 3.0).sin().abs() * 7.0; let c = cx.l(3, x, y);
                for q in 0..7 { let a = x / 6.0 + q as f32 * 0.9; let d = rot(vec2(6.4 * s, 0.0), a); seg(c - d, c + d, 0.55 * s, hx(0x8a6a38, 0.9), hx(0x8a6a38, 0.9)); }
                ring_e(c, 6.4 * s, 6.4 * s, 0.4 * s, hx(0x8a6a38, 0.9)); ring_e(c, 4.0 * s, 4.0 * s, 0.4 * s, hx(0x6a4a28, 0.8));
            }
            4 => {
                // cloud shadows sliding over the ground + heat shimmer
                for k in 0..2u32 { let x = (hash(k, 1) * 900.0 + t * 5.0) % 1200.0 - 50.0; fan(cx.l(4, x, 60.0), 110.0 * s, 18.0 * s, 14, hx(0x3a2410, 0.10), hx(0x3a2410, 0.0)); }
                additive(g, || { let (a, b) = (cx.w(0.0, 0.0), cx.w(0.0, 14.0)); let x0 = cx.w(cx.cam() - 260.0, 0.0).x; let x1 = cx.w(cx.cam() + 260.0, 0.0).x;
                    qd([vec2(x0, a.y), vec2(x1, a.y), vec2(x1, b.y), vec2(x0, b.y)], [hx(0xfff0c0, 0.10), hx(0xfff0c0, 0.10), hx(0xfff0c0, 0.0), hx(0xfff0c0, 0.0)]); });
            }
            _ => {}
        }
    }

    pub(super) fn plains_front(&self, cx: &Ctx, g: &Gfx) {
        let (s, t) = (cx.s, self.t);
        // dry grass tufts bending with the wind and the shock waves
        for k in 0..36u32 {
            let x = k as f32 * 16.0 + hash(k, 1) * 10.0; let p = cx.l(5, x, 240.0);
            if !vis(p, 40.0) { continue; }
            let wx = cx.lw(5, x);
            for b in 0..6u32 {
                let hgt = (14.0 + hash(k * 7 + b, 2) * 22.0) * if k % 5 == 0 { 1.4 } else { 1.0 };
                let bend = (self.wind(wx, hash(k, 3) * 6.0 + b as f32 * 0.3) * 8.0 + self.push_at(wx) * 24.0) * hgt / 24.0;
                let sp = (b as f32 - 2.5) * 1.6;
                let (base, tip) = (p + vec2(sp * s, 0.0), p + vec2((sp * 1.8 + bend) * s, -hgt * s));
                let mid = (base + tip) * 0.5 + vec2(bend * 0.18 * s, 0.0);
                let (c0, c1) = (hx(0x5a4a22, 1.0), if k % 2 == 0 { hx(0xe0c060, 1.0) } else { hx(0xc8a848, 1.0) });
                tr([base - vec2(1.1 * s, 0.0), base + vec2(1.1 * s, 0.0), mid], [c0, c0, lerpc(c0, c1, 0.6)]); tr([mid - vec2(0.8 * s, 0.0), mid + vec2(0.8 * s, 0.0), tip], [lerpc(c0, c1, 0.6), lerpc(c0, c1, 0.6), c1]);
            }
        }
        additive(g, || {
            fan(cx.v(cx.view().0 * 0.85, 8.0), 130.0 * s, 100.0 * s, 16, hx(0xfff0c0, 0.20), hx(0xfff0c0, 0.0));
            for q in 0..3 { let f = 0.25 + q as f32 * 0.22; fan(cx.v(cx.view().0 * (0.85 - f * 0.9), 8.0 + f * 150.0), (8.0 + q as f32 * 4.0) * s, (8.0 + q as f32 * 4.0) * s, 6, hx(0xffe0a0, 0.07), hx(0xffe0a0, 0.0)); }
        });
        let _ = t;
    }

    // ───────────────────────────────────────── SPIRIT ─────────────────────────────────────────
    pub(super) fn spirit_layer(&self, i: usize, cx: &Ctx, g: &Gfx) {
        let (s, t) = (cx.s, self.t);
        match i {
            0 => additive(g, || {
                for (k, col) in [hx(0x3affc0, 1.0), hx(0x4a90ff, 1.0), hx(0xb060ff, 1.0)].into_iter().enumerate() {
                    let (mut top, mut mid, mut bot, mut c0, mut c1) = (vec![], vec![], vec![], vec![], vec![]);
                    for j in 0..=26 {
                        let x = j as f32 * 20.0;
                        let yc = 50.0 + k as f32 * 18.0 + (x * 0.018 + t * (0.25 + k as f32 * 0.07) + k as f32 * 2.0).sin() * 14.0 + (x * 0.045 - t * 0.4).sin() * 5.0;
                        let hh = 16.0 + 6.0 * (x * 0.03 + t * 0.2 + k as f32).sin();
                        top.push(cx.l(0, x, yc - hh)); mid.push(cx.l(0, x, yc)); bot.push(cx.l(0, x, yc + hh * 0.6));
                        c0.push(ca(col, 0.0)); c1.push(ca(col, (0.14 + 0.05 * (x * 0.02 + t).sin()) * (1.0 + self.flicker * 1.5)));
                    }
                    strip(&top, &mid, &c0, &c1); strip(&mid, &bot, &c1, &c0);
                }
                let ph = (t % 11.0) / 1.2;
                if ph < 1.0 { let a = cx.l(0, 420.0 - ph * 280.0, 20.0 + ph * 70.0); let b = cx.l(0, 420.0 - ph * 280.0 + 40.0, 20.0 + ph * 70.0 - 14.0); seg(a, b, 1.4 * s, ca(hx(0xffffff, 1.0), 1.0 - ph), ca(hx(0xa0fff0, 1.0), 0.0)); }
            }),
            1 => { for k in 0..3u32 { let x = (hash(k, 1) * 900.0 + t * (3.0 + hash(k, 2) * 3.0)) % 900.0 - 80.0; fan(cx.l(1, x, 196.0 - hash(k, 3) * 24.0), 130.0 * s, 16.0 * s, 14, hx(0xb8fff4, 0.13), hx(0xb8fff4, 0.0)); } }
            2 => additive(g, || {
                let c = cx.l(2, 400.0, 152.0);
                let boost = 1.0 + self.cheer * 1.6 + self.flicker * 1.2;
                fan(c, 100.0 * s, 70.0 * s, 20, hx(0x70fff0, 0.13 * boost), hx(0x70fff0, 0.0));
                for ring in 0..3 {
                    let r = 28.0 + ring as f32 * 20.0; let dir = if ring % 2 == 0 { 1.0 } else { -1.0 };
                    for d in 0..18 { let a0 = t * (0.4 + ring as f32 * 0.2) * dir + d as f32 / 18.0 * 6.283;
                        let (p, q) = (c + vec2(a0.cos() * r * s * 0.95, a0.sin() * r * s * 1.1), c + vec2((a0 + 0.2).cos() * r * s * 0.95, (a0 + 0.2).sin() * r * s * 1.1));
                        seg(p, q, 1.1 * s, ca(hx(0x8affee, 1.0), 0.32 * boost.min(2.0)), ca(hx(0x8affee, 1.0), 0.32 * boost.min(2.0))); }
                }
                // glowing glyphs on the pillars
                for sx in [-98.0f32, 98.0] { for j in 0..4u32 { let y = 108.0 + j as f32 * 17.0; let b = (0.35 + 0.35 * (t * 1.4 + j as f32 + sx).sin()) * boost;
                    let p = cx.l(2, 400.0 + sx, y); let col = ca(hx(0x7affee, 1.0), b.min(1.0));
                    seg(p - vec2(3.0 * s, 0.0), p + vec2(3.0 * s, 0.0), 0.9 * s, col, col); seg(p + vec2(0.0, -3.0 * s), p + vec2(0.0, 3.0 * s), 0.9 * s, col, col);
                    fan(p, 7.0 * s, 7.0 * s, 8, ca(hx(0x7affee, 1.0), 0.18 * b.min(1.0)), ca(hx(0x7affee, 1.0), 0.0)); } }
            }),
            3 => {
                additive(g, || {
                    for (n, x) in toro_xs().into_iter().enumerate() {
                        let c = cx.l(3, x, 176.0);
                        if !vis(c, 60.0) { continue; }
                        let fl = self.flick(200 + n as u32);
                        fan(c, 27.0 * s, 27.0 * s, 14, hx(0x6affe8, 0.36 * fl), hx(0x6affe8, 0.0));
                        fan(c, 8.0 * s, 8.0 * s, 8, hx(0xffffff, 0.5 * fl), hx(0xffffff, 0.0));
                    }
                    // wandering will-o'-the-wisps with fading trails
                    for k in 0..7u32 {
                        for tr_ in 0..6u32 {
                            let tt = t - tr_ as f32 * 0.12;
                            let x = hash(k, 1) * 1000.0 + (tt * 0.35 + k as f32).sin() * 40.0 + (tt * 6.0 * (0.3 + hash(k, 5))) % 1000.0 * 0.0;
                            let y = 110.0 + hash(k, 2) * 70.0 + (tt * 0.6 + k as f32 * 2.0).sin() * 26.0 - (tt * (3.0 + hash(k, 3) * 4.0)) % 40.0;
                            fan(cx.l(3, x, y), (5.0 - tr_ as f32 * 0.7) * s, (5.0 - tr_ as f32 * 0.7) * s, 8, hx(0x9afff0, 0.55 * (1.0 - tr_ as f32 / 6.0)), hx(0x9afff0, 0.0));
                        }
                    }
                });
                // tiny forest spirits (kodama) that bounce when the fight heats up
                for (n, x) in toro_xs().into_iter().enumerate().filter(|(n, _)| n % 2 == 0) {
                    let ph = n as f32 * 1.7; let jump = self.cheer * (0.5 + 0.5 * (t * 9.0 + ph).sin().abs()) * 7.0 + (t * 1.4 + ph).sin().max(0.0) * 0.5;
                    let c = cx.l(3, x + 18.0, 212.0 - jump); let tilt = (t * 1.2 + ph).sin() * 0.6 * s;
                    crate::poly::circle(c + vec2(tilt, -4.0 * s), 3.6 * s, hx(0xe8fff8, 1.0));
                    qd([c + vec2(-1.8 * s, 0.0), c + vec2(1.8 * s, 0.0), c + vec2(2.4 * s, -3.0 * s), c + vec2(-2.4 * s, -3.0 * s)], [hx(0xd0f0e8, 1.0); 4]);
                    for e in [-1.2f32, 1.2] { crate::poly::circle(c + vec2(tilt + e * s, -4.6 * s), 0.55 * s, hx(0x102028, 1.0)); }
                    crate::poly::circle(c + vec2(tilt, -3.0 * s), 0.5 * s, hx(0x102028, 1.0));
                }
            }
            4 => {
                // carved rune rings breathe; they flare on supers and heavy hits
                let boost = 1.0 + self.cheer * 2.2 + self.flicker * 1.6;
                additive(g, || {
                    for k in 0..4 { let c = cx.l(4, 120.0 + k as f32 * 290.0, 74.0);
                        if !vis(c, 120.0 * s) { continue; }
                        let a = (0.12 + 0.12 * (t * 1.7 + k as f32 * 1.3).sin()) * boost;
                        ring_e(c, 52.0 * s, 11.0 * s, 0.8 * s, hx(0x5affea, a.min(0.8))); ring_e(c, 36.0 * s, 7.5 * s, 0.7 * s, hx(0x5affea, (a * 0.9).min(0.8)));
                        for q in 0..12 { let ang = q as f32 / 12.0 * 6.283 + t * 0.2; let p = c + vec2(ang.cos() * 44.0 * s, ang.sin() * 9.0 * s); fan(p, 3.0 * s, 1.5 * s, 6, hx(0x7affee, a.min(0.7)), hx(0x7affee, 0.0)); }
                        fan(c, 56.0 * s, 12.0 * s, 16, hx(0x5affea, 0.07 * boost.min(3.0)), hx(0x5affea, 0.0));
                    }
                    for (n, x) in toro_xs().into_iter().enumerate() { let p = vec2(cx.l(3, x, 0.0).x, cx.w(0.0, 8.0).y); if vis(p, 80.0) { fan(p, 40.0 * s, 7.0 * s, 12, hx(0x5affea, 0.10 * self.flick(200 + n as u32)), hx(0x5affea, 0.0)); } }
                });
                // ground mist rolling past
                for k in 0..5u32 { let x = (hash(k, 1) * 1100.0 + t * (7.0 + hash(k, 2) * 6.0)) % 1300.0 - 100.0; fan(cx.l(4, x, 44.0 + hash(k, 3) * 24.0), 130.0 * s, 13.0 * s, 16, hx(0xb8fff4, 0.10), hx(0xb8fff4, 0.0)); }
            }
            _ => {}
        }
    }

    pub(super) fn spirit_front(&self, cx: &Ctx, g: &Gfx) {
        let (s, t) = (cx.s, self.t);
        // hanging strings of paper charms (ofuda) under the top edge
        for (n, x0) in [20.0f32, 400.0].iter().enumerate() {
            let rope_y = |f: f32| 6.0 + 9.0 * 4.0 * f * (1.0 - f) + self.swing * 4.0 * (f * 3.14).sin();
            let mut prev: Option<Vec2> = None;
            for k in 0..=10 { let f = k as f32 / 10.0; let p = cx.l(5, x0 + f * 140.0, rope_y(f)); if let Some(q) = prev { seg(q, p, 0.8 * s, hx(0xc8b88a, 1.0), hx(0xc8b88a, 1.0)); } prev = Some(p); }
            for k in 1..8u32 { let f = k as f32 / 8.0; let top = cx.l(5, x0 + f * 140.0, rope_y(f));
                let amp = 1.0 + self.wind(*x0, k as f32 + n as f32) * 3.0 + self.shake * 2.5;
                banner(top, s, 20.0, 5.0, 5, amp, t, k as f32 * 1.3 + n as f32, hx(0xf4f0d8, 1.0), hx(0xd8d4b0, 1.0), Some(hx(0xc02a2a, 0.9))); }
        }
        additive(g, || {
            fan(cx.v(cx.view().0 * 0.5, cx.view().1), 200.0 * s, 45.0 * s, 16, hx(0x70ffe8, 0.08 + 0.05 * self.cheer), hx(0x70ffe8, 0.0));
        });
    }

    // ───────────────────────────────────────── VOLCANO ─────────────────────────────────────────
    pub(super) fn volcano_layer(&self, i: usize, cx: &Ctx, g: &Gfx) {
        let (s, t) = (cx.s, self.t);
        match i {
            0 => additive(g, || {
                let ph = t % 7.0; let fl = if ph < 0.22 { 1.0 - ph / 0.22 } else { 0.0 };
                let a = fl.max(self.flicker * 0.7);
                if a > 0.02 {
                    let c = cx.l(0, 200.0 + hash((t / 7.0) as u32, 1) * 180.0, 50.0);
                    fan(c, 230.0 * s, 90.0 * s, 20, hx(0xffa070, 0.3 * a), hx(0xff7030, 0.0));
                    if fl > 0.0 { let (mut x, mut y) = (c.x, cx.l(0, 0.0, 0.0).y); for k in 0..7 { let nx = x + (hash((t / 7.0) as u32, k + 10) - 0.5) * 30.0 * s; let ny = y + 20.0 * s; seg(vec2(x, y), vec2(nx, ny), 1.6 * s, ca(hx(0xfff0ff, 1.0), fl), ca(hx(0xc8a0ff, 1.0), fl)); x = nx; y = ny; } }
                }
            }),
            1 => {
                let crater = (418.0, 90.0);
                // smoke column rising from the crater, lit orange from below
                for k in 0..10u32 {
                    let p = (t * 0.1 + k as f32 / 10.0) % 1.0;
                    let c = cx.l(1, crater.0 + (p * 5.0 + k as f32).sin() * 12.0 + self.wind(0.0, 1.0) * 30.0 * p, crater.1 - p * 120.0);
                    let r = (7.0 + p * 30.0) * s;
                    fan(c, r, r * 0.85, 12, hx(0x24100c, 0.55 * (1.0 - p)), hx(0x24100c, 0.0));
                    if p < 0.4 { additive(g, || fan(c + vec2(0.0, r * 0.3), r * 0.8, r * 0.6, 10, hx(0xff6a20, 0.25 * (1.0 - p / 0.4)), hx(0xff6a20, 0.0))); }
                }
                // periodic lava fountain (always erupting a little more when the arena shakes)
                additive(g, || {
                    let e = t % 5.0; let boost = 1.0 + self.shake * 1.5;
                    if e < 2.4 { for q in 0..(26.0 * boost) as u32 {
                        let ang = -1.57 + (hash(q, 3) - 0.5) * 1.0; let sp = 50.0 + hash(q, 4) * 46.0 * boost;
                        let (x, y) = (crater.0 + ang.cos() * sp * e, crater.1 + ang.sin() * sp * e + 0.5 * 52.0 * e * e);
                        fan(cx.l(1, x, y), 2.6 * s, 2.6 * s, 8, hx(0xffc060, 0.9 * (1.0 - e / 2.4)), hx(0xff6a20, 0.0)); } }
                    fan(cx.l(1, crater.0, crater.1), 46.0 * s, 20.0 * s, 16, hx(0xff8a30, 0.22 * self.flick(900)), hx(0xff8a30, 0.0));
                });
            }
            2 => additive(g, || {
                for k in 0..34u32 {
                    let x = (hash(k, 1) * 1000.0 + t * (8.0 + hash(k, 2) * 12.0)) % 1000.0; let y = 201.0 + hash(k, 3) * 10.0; let l = 12.0 + hash(k, 4) * 22.0;
                    let a = 0.25 + 0.3 * (t * 2.0 + k as f32).sin();
                    seg(cx.l(2, x, y), cx.l(2, x + l, y + 0.3), 1.4 * s, hx(0xffe080, a.max(0.0)), hx(0xff8a20, 0.0));
                }
                for k in 0..9u32 { let p = (t * 0.45 + hash(k, 1)) % 1.0; let c = cx.l(2, hash(k, 2) * 1000.0, 205.0 + hash(k, 3) * 6.0); ring_e(c, (1.0 + p * 5.0) * s, (1.0 + p * 2.5) * s, 0.5 * s, hx(0xffb040, 0.7 * (1.0 - p))); }
            }),
            3 => additive(g, || {
                for k in 0..22u32 {
                    let p = (t * 0.22 + hash(k, 1)) % 1.0; let x = hash(k, 2) * 1100.0 + (p * 8.0 + k as f32).sin() * 7.0; let c = cx.l(3, x, 208.0 - p * 130.0);
                    fan(c, 3.0 * s, 3.0 * s, 6, hx(0xffa040, 0.9 * (1.0 - p)), hx(0xff5010, 0.0));
                }
                for k in 0..6 { fan(cx.l(3, 80.0 + k as f32 * 190.0, 214.0), 70.0 * s, 24.0 * s, 14, hx(0xff6a20, 0.18 * self.flick(910 + k)), hx(0xff6a20, 0.0)); }
            }),
            4 => additive(g, || {
                let hot = 1.0 + self.flicker * 1.5 + self.shake * 0.6;
                for (n, pl) in lava_cracks(1100.0).iter().enumerate() {
                    let c0 = cx.l(4, pl[0].0, pl[0].1);
                    if !vis(c0, 200.0 * s) { continue; }
                    let a = ((0.25 + 0.28 * (t * 1.8 + n as f32).sin()) * hot).clamp(0.0, 0.95);
                    for w in pl.windows(2) { let (p, q) = (cx.l(4, w[0].0, w[0].1), cx.l(4, w[1].0, w[1].1)); seg(p, q, 2.6 * s, hx(0xff6a20, a * 0.5), hx(0xff6a20, a * 0.5)); seg(p, q, 0.9 * s, hx(0xffe090, a), hx(0xffe090, a)); }
                    fan(c0, 20.0 * s, 6.0 * s, 10, hx(0xff6a20, a * 0.25), hx(0xff6a20, 0.0));
                }
                // lava glow lapping at the far edge of the arena
                let (a, b) = (cx.w(0.0, -2.0), cx.w(0.0, 16.0)); let x0 = cx.w(cx.cam() - 260.0, 0.0).x; let x1 = cx.w(cx.cam() + 260.0, 0.0).x;
                let fl = 0.14 * self.flick(920);
                qd([vec2(x0, a.y), vec2(x1, a.y), vec2(x1, b.y), vec2(x0, b.y)], [hx(0xff7a20, fl), hx(0xff7a20, fl), hx(0xff7a20, 0.0), hx(0xff7a20, 0.0)]);
            }),
            _ => {}
        }
    }

    pub(super) fn volcano_front(&self, cx: &Ctx, g: &Gfx) {
        let (s, t) = (cx.s, self.t);
        let (vw, vh) = cx.view();
        // ash flakes drifting down-wind
        for k in 0..42u32 {
            let sp = 8.0 + hash(k, 2) * 14.0; let y = (t * sp + hash(k, 1) * vh) % (vh + 20.0) - 10.0;
            let x = ((hash(k, 3) * (vw + 80.0) - t * (6.0 + self.gust * 14.0) - cx.cam() * 0.6 + y * 0.2).rem_euclid(vw + 80.0)) - 40.0;
            let r = (0.5 + hash(k, 4) * 1.0) * s; crate::poly::circle(cx.v(x, y), r, hx(0x4a3a36, 0.55));
        }
        additive(g, || {
            // heat haze: faint wavy light bands
            for b in 0..4u32 { let y0 = 120.0 + b as f32 * 26.0; let (mut top, mut bot, mut c) = (vec![], vec![], vec![]);
                for j in 0..=18 { let x = j as f32 * vw / 18.0; let off = (x * 0.05 + t * 1.8 + b as f32).sin() * 3.0; top.push(cx.v(x, y0 + off)); bot.push(cx.v(x, y0 + 8.0 + off)); c.push(hx(0xff8a40, 0.03)); }
                strip(&top, &bot, &c, &c); }
            fan(cx.v(vw * 0.5, vh), vw * 0.4 * s, 40.0 * s, 16, hx(0xff6a20, 0.16 * self.flick(930)), hx(0xff6a20, 0.0));
            if self.flicker > 0.3 { qd([cx.v(0.0, 0.0), cx.v(vw, 0.0), cx.v(vw, vh), cx.v(0.0, vh)], [hx(0xff5010, 0.05 * self.flicker); 4]); }
        });
    }
}

const NEON_RGB: [u32; 5] = [0xff2fd0, 0x22e8ff, 0xffe23a, 0x7d5bff, 0x3dff8a];
#[allow(dead_code)]
fn _keep(_: &[&str; 5]) { let _ = NEON; let _ = nz(0.0, 0.0); }
