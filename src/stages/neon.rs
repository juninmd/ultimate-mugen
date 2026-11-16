use super::*;

mod layers_0;
mod layers_1;
mod layers_2;
mod tower_block;
use tower_block::*;

pub(super) fn neon() -> Stage {
    let signs = neon_signs();
    let mut back = layers_0::layers(&signs);
    back.extend(layers_1::layers());
    back.extend(layers_2::layers());
    let particles = parts(9, 24, |r| Particle {
        x: r.f() * 470.0,
        y: r.f() * 240.0,
        vx: -0.6,
        vy: 5.0 + r.f() * 2.0,
        s: 0.35,
        a: 0.35,
        c: [154, 216, 255],
    });
    Stage {
        info: info("Neo-Metrópole 2099"),
        art: StageArt::Procedural {
            back,
            particles,
            tint: None,
        },
        music: 1,
        kind: StageKind::Neon,
        bgm: None,
    }
}
