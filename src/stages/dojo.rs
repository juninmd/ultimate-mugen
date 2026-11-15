use super::*;

mod layers_0;
mod layers_1;
mod garden;
mod rooftop_deck;
use garden::*;
use rooftop_deck::*;

pub(super) fn dojo() -> Stage {
    let mut back = layers_0::layers();
    back.extend(layers_1::layers());
    let particles = parts(5, 24, |r| Particle {
        x: r.f() * 470.0,
        y: r.f() * 240.0,
        vx: 0.3 + r.f() * 0.5,
        vy: 0.25 + r.f() * 0.4,
        s: 0.8 + r.f() * 1.1,
        a: 0.6 + r.f() * 0.4,
        c: if r.f() < 0.5 {
            [255, 192, 216]
        } else {
            [255, 158, 192]
        },
    });
    Stage {
        info: info("Telhado do Dojo — Pôr do Sol"),
        art: StageArt::Procedural {
            back,
            particles,
            tint: Some([1.0, 0.55, 0.2, 0.04]),
        },
        music: 0,
        kind: StageKind::Dojo,
        bgm: None,
    }
}
