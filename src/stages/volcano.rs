use super::*;

mod layers_0;
mod layers_1;
mod basalt_column;
use basalt_column::*;

pub(super) fn volcano() -> Stage {
    let cracks = lava_cracks(1100.0);
    let mut back = layers_0::layers();
    back.extend(layers_1::layers(&cracks));
    let particles = parts(8, 36, |r| Particle {
        x: r.f() * 470.0,
        y: r.f() * 240.0,
        vx: (r.f() - 0.5) * 0.4,
        vy: -0.5 - r.f() * 0.8,
        s: 0.5 + r.f() * 0.8,
        a: 0.9,
        c: if r.f() < 0.5 {
            [255, 176, 48]
        } else {
            [255, 90, 16]
        },
    });
    Stage {
        info: info("Cratera Vulcânica"),
        art: StageArt::Procedural {
            back,
            particles,
            tint: Some([1.0, 0.24, 0.0, 0.05]),
        },
        music: 5,
        kind: StageKind::Volcano,
        bgm: None,
    }
}
