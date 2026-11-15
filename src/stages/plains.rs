use super::*;

mod layers_0;
mod layers_1;
mod rocks;
use rocks::*;

pub(super) fn plains() -> Stage {
    let mut back = layers_0::layers();
    back.extend(layers_1::layers());
    let particles = parts(4, 14, |r| Particle {
        x: r.f() * 470.0,
        y: 150.0 + r.f() * 80.0,
        vx: 0.8 + r.f(),
        vy: -0.05,
        s: 0.6,
        a: 0.4,
        c: [232, 200, 160],
    });
    Stage {
        info: info("Planalto Rochoso"),
        art: StageArt::Procedural {
            back,
            particles,
            tint: Some([1.0, 0.85, 0.5, 0.025]),
        },
        music: 3,
        kind: StageKind::Plains,
        bgm: None,
    }
}
