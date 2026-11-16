mod support;
use std::sync::Arc;
use support::*;

const CNS: &str = "[Statedef 0]\ntype = S\nmovetype = I\nphysics = S\n";

#[test]
fn changeanim2_with_unknown_anim_keeps_the_victims_own_sprites() {
    let atk = mugen_char("atk", CNS);
    let vic = mugen_char("vic", CNS);
    let mut m = duel(atk.clone(), vic.clone());
    ticks(&mut m, 2);
    let before = m.fighters[1].anim_no;
    m.fighters[1].change_anim(987_654, &atk, 1);
    let v = &m.fighters[1];
    assert!(
        Arc::ptr_eq(&v.anim_def, &v.def),
        "victim switched to the attacker's idle sprites for an animation the attacker does not have"
    );
    assert_eq!(v.anim_no, before);
}
