use super::*;

mod layers_0;
mod layers_1;

// ───────────────────────────────────────────── BAMBOO ─────────────────────────────────────────────

pub(super) fn bamboo() -> Stage {
    let mut back = layers_0::layers();
    back.extend(layers_1::layers());
    let particles = parts(3, 20, |r| Particle {
        x: r.f() * 470.0,
        y: 80.0 + r.f() * 140.0,
        vx: (r.f() - 0.5) * 0.3,
        vy: (r.f() - 0.5) * 0.2,
        s: 0.9,
        a: 0.8,
        c: [217, 255, 122],
    });
    Stage {
        info: info("Bambuzal ao Luar"),
        art: StageArt::Procedural {
            back,
            particles,
            tint: Some([0.16, 0.47, 0.55, 0.06]),
        },
        music: 2,
        kind: StageKind::Bamboo,
        bgm: None,
    }
}
