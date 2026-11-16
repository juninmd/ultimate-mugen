use super::*;

/// A cheering spectator standing at `feet`, `u` = pixels per unit. `jump` lifts the body, `arms` (0..1) raises the arms.
#[allow(clippy::too_many_arguments)]
pub(super) fn spectator(
    feet: Vec2,
    u: f32,
    h: f32,
    jump: f32,
    arms: f32,
    body: Color,
    head: Color,
    accent: Color,
    stick: Option<Color>,
    t: f32,
    ph: f32,
) {
    let b = feet - vec2(0.0, jump * u);
    let w = h * 0.34 * u;
    let hh = h * u;
    let sway = (t * 2.0 + ph).sin() * 0.6 * u;
    qd(
        [
            b + vec2(-w * 0.5 + sway, -hh * 0.78),
            b + vec2(w * 0.5 + sway, -hh * 0.78),
            b + vec2(w * 0.62, 0.0),
            b + vec2(-w * 0.62, 0.0),
        ],
        [accent, accent, body, body],
    );
    let hc = b + vec2(sway, -hh * 0.9);
    crate::poly::circle(hc, h * 0.16 * u, head);
    crate::poly::fill(
        &crate::poly::regular(hc + vec2(0.0, -h * 0.03 * u), h * 0.17 * u, 8, 0.0)
            .into_iter()
            .filter(|p| p.y <= hc.y - h * 0.03 * u + 0.1)
            .collect::<Vec<_>>(),
        |_| crate::poly::shade(body, 0.45),
    );
    // arms: down at rest, thrown up when cheering
    let al = hc + vec2(-w * 0.55, hh * 0.26);
    let ar = hc + vec2(w * 0.55, hh * 0.26);
    let wave = (t * 11.0 + ph * 3.0).sin() * 0.25 * arms;
    let tl = al
        + vec2(
            -w * (0.3 + 0.2 * arms),
            -hh * (0.1 + 0.62 * arms) - wave * hh * 0.2,
        );
    let tr_ = ar
        + vec2(
            w * (0.3 + 0.2 * arms),
            -hh * (0.1 + 0.62 * arms) + wave * hh * 0.2,
        );
    seg(al, tl, 0.18 * h * u * 0.6, body, body);
    seg(ar, tr_, 0.18 * h * u * 0.6, body, body);
    if let Some(sc) = stick {
        if arms > 0.15 {
            seg(
                tr_,
                tr_ + vec2(wave * 4.0 * u, -h * 0.34 * u),
                0.2 * u * 1.4,
                ca(sc, 0.9 * arms),
                ca(sc, 0.0),
            );
            seg(
                tl,
                tl + vec2(-wave * 4.0 * u, -h * 0.34 * u),
                0.2 * u * 1.4,
                ca(sc, 0.9 * arms),
                ca(sc, 0.0),
            );
        }
    }
}

/// Hanging cloth strip (flag / banner / paper charm). `top` anchor, `len` hangs downward along +y, `width` across.
#[allow(clippy::too_many_arguments)]
pub(super) fn banner(
    top: Vec2,
    u: f32,
    len: f32,
    width: f32,
    segs: usize,
    amp: f32,
    t: f32,
    ph: f32,
    c_top: Color,
    c_bot: Color,
    mark: Option<Color>,
) {
    let (mut a, mut b, mut ca_, mut cb) = (vec![], vec![], vec![], vec![]);
    for k in 0..=segs {
        let f = k as f32 / segs as f32;
        let off = (t * 5.0 - f * 3.2 + ph).sin() * amp * f * u + amp * f * f * u * 0.5;
        let slope = ((t * 5.0 - f * 3.2 + ph).cos() * 0.5 + 0.5) * 0.35;
        let y = f * len * u * (1.0 - 0.06 * (off / u).abs() / amp.max(0.1));
        a.push(top + vec2(off - width * 0.5 * u, y));
        b.push(top + vec2(off + width * 0.5 * u, y));
        let c = lerpc(c_top, c_bot, f);
        let sh = 1.0 - slope * 0.6 + slope * 0.2;
        ca_.push(crate::poly::shade(c, sh * 1.1));
        cb.push(crate::poly::shade(c, sh * 0.8));
    }
    strip(&a, &b, &ca_, &cb);
    if let Some(mc) = mark {
        let (mut a2, mut b2, mut c2) = (vec![], vec![], vec![]);
        for k in 2..=segs.saturating_sub(1).max(3) {
            let i = k.min(segs);
            a2.push((a[i] * 0.72) + (b[i] * 0.28));
            b2.push((a[i] * 0.28) + (b[i] * 0.72));
            c2.push(mc);
        }
        strip(&a2, &b2, &c2, &c2);
    }
}
