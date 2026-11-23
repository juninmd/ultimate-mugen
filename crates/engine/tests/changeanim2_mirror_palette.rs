use crate::support;
use std::sync::Arc;
use support::*;

const CNS: &str = "[Statedef 0]\ntype = S\nmovetype = I\nphysics = S\n";

#[test]
fn changeanim2_in_a_mirror_match_keeps_the_victims_own_palette() {
    let atk = mugen_char("widow", CNS);
    let vic = mugen_char("widow", CNS);
    let mut m = duel(atk.clone(), vic.clone());
    ticks(&mut m, 2);
    m.fighters[1].change_anim(5000, &atk, 1);
    assert!(Arc::ptr_eq(&m.fighters[1].anim_def, &m.fighters[1].def), "victim drew the attacker's palette in a mirror match");
}

#[test]
fn changeanim2_between_different_fighters_still_plays_the_attackers_art() {
    let atk = mugen_char("atk", CNS);
    let vic = mugen_char("vic", CNS);
    let mut m = duel(atk.clone(), vic.clone());
    ticks(&mut m, 2);
    m.fighters[1].change_anim(5000, &atk, 1);
    assert!(Arc::ptr_eq(&m.fighters[1].anim_def, &atk));
}
