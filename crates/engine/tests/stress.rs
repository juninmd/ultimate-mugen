mod support;
use std::time::Instant;
use support::*;

const SPAM: &str = r#"
[Statedef 300]
type = S
movetype = I
physics = S
anim = 0
ctrl = 0

[State 300, e]
type = Explod
trigger1 = 1
id = 1
anim = 1000
pos = 0, 0
removetime = -1

[State 300, p]
type = Projectile
trigger1 = 1
projanim = 1000
attr = S, NP
damage = 1
projremovetime = -1
ground.velocity = 0

[State 300, h]
type = Helper
trigger1 = 1
stateno = 300
"#;

#[test]
fn spawners_stay_bounded() {
    let c = mugen_char("spam", SPAM);
    let mut m = duel(c.clone(), c.clone());
    m.start_round();
    enter(&mut m, 0, 300);
    let t = Instant::now();
    for k in 0..2000 {
        m.tick(&[0, 0]);
        if k % 400 == 399 {
            println!("tick {k}: {} ms, explods {}, projectiles {}, fighters {}", t.elapsed().as_millis(), m.explods.len(), m.projectiles.len(), m.fighters.len());
        }
    }
    let ms = t.elapsed().as_millis();
    println!("2000 ticks spam: {ms} ms, explods {}, projectiles {}, fighters {}", m.explods.len(), m.projectiles.len(), m.fighters.len());
    assert!(m.explods.len() <= 256, "explods unbounded: {}", m.explods.len());
    assert!(m.projectiles.len() <= 256, "projectiles unbounded: {}", m.projectiles.len());
    assert!(m.fighters.len() <= 64);
}
