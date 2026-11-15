use super::*;

mod layers_0;
mod layers_1;
mod shrine;
use shrine::*;

pub(super) fn spirit() -> Stage {
    let mut back = layers_0::layers();
    back.extend(layers_1::layers());
    let particles = parts(6, 22, |r| Particle {
        x: r.f() * 470.0,
        y: r.f() * 240.0,
        vx: (r.f() - 0.5) * 0.2,
        vy: -0.3 - r.f() * 0.4,
        s: 0.7 + r.f(),
        a: 0.5,
        c: [184, 255, 240],
    });
    Stage {
        info: info("Portão dos Espíritos"),
        art: StageArt::Procedural {
            back,
            particles,
            tint: None,
        },
        music: 4,
        kind: StageKind::Spirit,
        bgm: None,
    }
}
