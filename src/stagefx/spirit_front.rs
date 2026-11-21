use super::*;

impl StageReact {
    pub(super) fn spirit_front(&self, cx: &Ctx, g: &Gfx) {
        let (s, t) = (cx.s, self.t);
        for (n, x0) in [20.0f32, 400.0].iter().enumerate() {
            let rope_y = |f: f32| {
                6.0 + 9.0 * 4.0 * f * (1.0 - f)
                    + self.swing * 4.0 * (f * std::f32::consts::PI).sin()
            };
            let mut prev: Option<Vec2> = None;
            for k in 0..=10 {
                let f = k as f32 / 10.0;
                let p = cx.l(5, x0 + f * 140.0, rope_y(f));
                if let Some(q) = prev {
                    seg(q, p, 0.8 * s, hx(0xc8b88a, 1.0), hx(0xc8b88a, 1.0));
                }
                prev = Some(p);
            }
            for k in 1..8u32 {
                let f = k as f32 / 8.0;
                let top = cx.l(5, x0 + f * 140.0, rope_y(f));
                let amp = 1.0 + self.wind(*x0, k as f32 + n as f32) * 3.0 + self.shake * 2.5;
                let phase = k as f32 * 1.3 + n as f32;
                let angle = self.swing * 0.2 + (t * 5.0 + phase).sin() * amp * 0.08;
                if !spirit_charm::draw(top, 7.0 * s, 25.0 * s, angle) {
                    banner(
                        top,
                        s,
                        20.0,
                        5.0,
                        5,
                        amp,
                        t,
                        phase,
                        hx(0xf4f0d8, 1.0),
                        hx(0xd8d4b0, 1.0),
                        Some(hx(0xc02a2a, 0.9)),
                    );
                }
            }
        }
        additive(g, || {
            fan(
                cx.v(cx.view().0 * 0.5, cx.view().1),
                200.0 * s,
                45.0 * s,
                16,
                hx(0x70ffe8, 0.08 + 0.05 * self.cheer),
                hx(0x70ffe8, 0.0),
            );
        });
    }
}
