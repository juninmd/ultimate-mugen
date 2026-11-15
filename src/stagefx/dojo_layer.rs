use super::*;

impl StageReact {
    pub(super) fn dojo_layer(&self, i: usize, cx: &Ctx, g: &Gfx) {
        let (s, t) = (cx.s, self.t);
        match i {
            0 => additive(g, || {
                // god rays fanning out of the setting sun
                let c = cx.l(0, 260.0, 150.0);
                for k in 0..7 {
                    let a = -std::f32::consts::PI
                        + (k as f32 + 0.5) / 7.0 * std::f32::consts::PI
                        + (t * 0.05 + k as f32).sin() * 0.04;
                    let wd = 0.07 + 0.03 * (t * 0.4 + k as f32 * 2.0).sin();
                    let (p1, p2) = (
                        c + vec2((a - wd).cos(), (a - wd).sin()) * 260.0 * s,
                        c + vec2((a + wd).cos(), (a + wd).sin()) * 260.0 * s,
                    );
                    tr(
                        [c, p1, p2],
                        [
                            hx(0xffe8b0, 0.085 + 0.025 * (t + k as f32).sin()),
                            hx(0xffb070, 0.0),
                            hx(0xffb070, 0.0),
                        ],
                    );
                }
            }),
            1 => {
                // flock of birds crossing the sky, wings flapping
                for k in 0..7u32 {
                    let x = (t * 16.0 + k as f32 * 21.0) % 820.0 - 60.0;
                    let y = 70.0 + (k as f32 * 1.9).sin() * 16.0 + (t * 0.8 + k as f32).sin() * 3.0;
                    let flap = (t * 8.0 + k as f32 * 1.3).sin() * (0.45 + self.gust * 0.2);
                    let c = cx.l(1, x, y);
                    let col = hx(0x2a1030, 0.9);
                    let (l, r) = (
                        c + vec2(-5.5, -2.5 * flap - 0.5) * s,
                        c + vec2(5.5, -2.5 * flap - 0.5) * s,
                    );
                    seg(l, c, 1.1 * s, col, col);
                    seg(c, r, 1.1 * s, col, col);
                }
            }
            2 => {
                // chimney smoke from the village + lit windows twinkling
                for ch in 0..2 {
                    let bx = 180.0 + ch as f32 * 470.0;
                    for k in 0..6 {
                        let p = (t * 0.22 + k as f32 / 6.0 + ch as f32 * 0.3) % 1.0;
                        let c = cx.l(
                            2,
                            bx + p * 30.0 * (0.4 + self.wind(bx, 1.0))
                                + (p * 9.0 + k as f32).sin() * 3.0,
                            178.0 - p * 60.0,
                        );
                        fan(
                            c,
                            (3.0 + p * 11.0) * s,
                            (3.0 + p * 11.0) * s,
                            10,
                            hx(0x6a3a5a, 0.30 * (1.0 - p)),
                            hx(0x6a3a5a, 0.0),
                        );
                    }
                }
                additive(g, || {
                    for k in 0..22u32 {
                        let (x, y) = (30.0 + hash(k, 5) * 840.0, 170.0 + hash(k, 6) * 36.0);
                        let b = 0.3 + 0.7 * ((t * (0.5 + hash(k, 7)) + k as f32).sin() * 0.5 + 0.5);
                        fan(
                            cx.l(2, x, y),
                            4.0 * s,
                            4.0 * s,
                            8,
                            hx(0xffc060, 0.5 * b),
                            hx(0xffc060, 0.0),
                        );
                    }
                });
            }
            3 => {
                // students watching behind the railing; they jump and cheer on supers / KOs
                for k in 0..20u32 {
                    let x = 40.0 + k as f32 * 48.0 + hash(k, 1) * 14.0;
                    let ph = hash(k, 2) * std::f32::consts::TAU;
                    let jump = self.cheer * (0.5 + 0.5 * (t * 9.0 + ph).sin().abs()) * 8.0
                        + (t * 1.7 + ph).sin().max(0.0) * 0.4;
                    let base = [
                        hx(0xe8e0d8, 1.0),
                        hx(0xd0c8c0, 1.0),
                        hx(0xf0e8e0, 1.0),
                        hx(0x40306a, 1.0),
                    ][(hash(k, 3) * 4.0) as usize % 4];
                    let dark = crate::poly::shade(base, 0.55);
                    let head = hx(0xe8b898, 1.0);
                    let tint = hx(0xff8a60, 1.0);
                    let (base, dark) =
                        (lerpc(base, tint, 0.35), lerpc(dark, hx(0x6a2a4a, 1.0), 0.4));
                    let c = cx.l(3, x, 207.0);
                    spectator(
                        c,
                        s,
                        31.0,
                        jump,
                        (self.cheer * 1.4).min(1.0),
                        dark,
                        head,
                        base,
                        None,
                        t,
                        ph,
                    );
                    if hash(k, 4) < 0.6 {
                        let hb = c - vec2(0.0, (31.0 * 0.9 + jump) * s);
                        seg(
                            hb + vec2(-4.5 * s, 0.0),
                            hb + vec2(4.5 * s, 0.0),
                            1.2 * s,
                            hx(0xd02a2a, 1.0),
                            hx(0xd02a2a, 1.0),
                        );
                    }
                }
                // falling blossoms on the sakura trees
                for k in 0..24u32 {
                    let sp = 5.0 + hash(k, 8) * 7.0;
                    let y = (t * sp + hash(k, 9) * 240.0) % 250.0 - 10.0;
                    let x = (hash(k, 10) * 1000.0
                        + (t * 0.8 + k as f32).sin() * 14.0
                        + t * (2.0 + self.wind(0.0, k as f32) * 8.0))
                        % 1000.0;
                    let c = cx.l(3, x, y);
                    let a = t * 2.0 + k as f32;
                    let d = rot(vec2(2.0 * s, 0.0), a);
                    tr(
                        [c - d, c + vec2(-d.y, d.x) * 0.5, c + d],
                        [hx(0xffc0d0, 0.9), hx(0xff90b0, 0.9), hx(0xffc0d0, 0.9)],
                    );
                }
            }
            4 => {
                // lantern light pools on the deck + halos
                additive(g, || {
                    for (n, x) in dojo_lanterns().into_iter().enumerate() {
                        if !(cx.l(4, x, 12.0).x > -80.0
                            && cx.l(4, x, 12.0).x < screen_width() + 80.0)
                        {
                            continue;
                        }
                        let fl = self.flick(n as u32);
                        fan(
                            cx.l(4, x, 12.0),
                            22.0 * s,
                            24.0 * s,
                            14,
                            hx(0xffa050, 0.30 * fl),
                            hx(0xff7030, 0.0),
                        );
                        fan(
                            cx.l(4, x, 46.0),
                            46.0 * s,
                            10.0 * s,
                            16,
                            hx(0xffa050, 0.13 * fl),
                            hx(0xff7030, 0.0),
                        );
                    }
                    // sunset shafts across the planks
                    for k in 0..4 {
                        let x0 = cx.cam() * 0.0
                            + 120.0
                            + k as f32 * 290.0
                            + (t * 0.3 + k as f32).sin() * 12.0;
                        qd(
                            [
                                cx.l(4, x0, 30.0),
                                cx.l(4, x0 + 38.0, 30.0),
                                cx.l(4, x0 + 150.0, 118.0),
                                cx.l(4, x0 + 90.0, 118.0),
                            ],
                            [
                                hx(0xffc880, 0.10),
                                hx(0xffc880, 0.10),
                                hx(0xffb070, 0.0),
                                hx(0xffb070, 0.0),
                            ],
                        );
                    }
                });
            }
            _ => {}
        }
    }
}
