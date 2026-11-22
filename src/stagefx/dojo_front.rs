use super::*;

impl StageReact {
    pub(super) fn dojo_front(&self, cx: &Ctx, g: &Gfx) {
        let (s, t) = (cx.s, self.t);
        // big paper lanterns swinging under the eave (layer 5)
        for k in 0..3u32 {
            let x = 70.0 + k as f32 * 210.0;
            let rope = 22.0 + hash(k, 1) * 8.0;
            let ang = self.swing * 0.55
                + (t * 1.3 + k as f32 * 1.9).sin() * (0.03 + self.gust * 0.015)
                + self.shake * (t * 33.0 + k as f32).sin() * 0.05;
            let top = cx.l(5, x, 30.0);
            let c = top + rot(vec2(0.0, rope * s), -ang);
            let fl = self.flick(100 + k);
            seg(top, c, 0.6 * s, hx(0x1a0c0a, 1.0), hx(0x1a0c0a, 1.0));
            let (rx, ry) = (11.0 * s, 15.0 * s);
            let warm = Color::new(0.94 + 0.06 * fl, 0.82 + 0.18 * fl, 0.75 + 0.25 * fl, 1.0);
            if !dojo_lantern::draw(c + vec2(0.0, ry), rx * 2.0, ry * 2.0, warm) {
                fan(
                    c + vec2(0.0, ry),
                    rx,
                    ry,
                    16,
                    lerpc(
                        hx(0xffd890, 1.0),
                        hx(0xff6a30, 1.0),
                        0.45 * (1.0 - fl * 0.5),
                    ),
                    hx(0xb81c18, 1.0),
                );
                for r in 1..5 {
                    let f = r as f32 / 5.0 - 0.1;
                    let yy = (f - 0.5) * 2.0;
                    let w = (1.0 - yy * yy).sqrt() * rx;
                    seg(
                        c + vec2(-w, ry + yy * ry * 0.98),
                        c + vec2(w, ry + yy * ry * 0.98),
                        0.5 * s,
                        hx(0x701410, 0.6),
                        hx(0x701410, 0.6),
                    );
                }
                qd(
                    [
                        c + vec2(-rx * 0.5, 0.0),
                        c + vec2(rx * 0.5, 0.0),
                        c + vec2(rx * 0.5, 1.6 * s),
                        c + vec2(-rx * 0.5, 1.6 * s),
                    ],
                    [hx(0x140808, 1.0); 4],
                );
            }
            seg(
                c + vec2(0.0, 2.0 * ry),
                c + vec2(0.0, 2.0 * ry + 6.0 * s),
                0.7 * s,
                hx(0xe8c070, 1.0),
                hx(0xa07030, 1.0),
            );
            additive(g, || {
                fan(
                    c + vec2(0.0, ry),
                    30.0 * s,
                    30.0 * s,
                    14,
                    hx(0xff9040, 0.22 * fl),
                    hx(0xff7030, 0.0),
                )
            });
        }
        // nobori banners on tall poles at the screen edges (layer 6): flap harder in gusts / hits
        for (k, x) in [38.0f32, 520.0].iter().enumerate() {
            let top = cx.l(6, *x, 60.0);
            let pole_c = hx(0x2a1410, 1.0);
            seg(
                cx.l(6, *x, 250.0),
                cx.l(6, *x, 56.0),
                2.4 * s,
                pole_c,
                pole_c,
            );
            seg(
                top + vec2(0.0, 2.0 * s),
                top + vec2(18.0 * s, 2.0 * s),
                1.4 * s,
                pole_c,
                pole_c,
            );
            let amp = 2.0 + self.wind(*x, k as f32) * 5.0 + self.shake * 4.0;
            banner(
                top + vec2(0.0, 3.0 * s),
                s,
                88.0,
                17.0,
                10,
                amp,
                t,
                k as f32 * 1.7,
                hx(0xc42a2a, 1.0),
                hx(0x8a1818, 1.0),
                Some(hx(0xf4ece0, 0.95)),
            );
        }
    }
}
