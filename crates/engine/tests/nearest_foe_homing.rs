mod support;
use std::sync::Arc;
use support::*;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::matchs::Match;

const SPAWN: &str = "
[Statedef 3000]
type = S
movetype = I
physics = N
anim = 0
ctrl = 0

[State 3000, dragon]
type = Helper
trigger1 = Time = 0
id = 3050
stateno = 3050
pos = 30, -40
postype = p1
facing = 1
";

const DRAGON: &str = include_str!("../../../tools/mugen2umf/overlays/goku_ssj4_ryuken.cns");

fn dragon_char(name: &str) -> Arc<ume_engine::chardef::CharDef> {
    let mut d = mugen_char(name, &format!("{SPAWN}\n{DRAGON}"));
    let def = Arc::get_mut(&mut d).unwrap();
    def.anims.insert(3050, anim(3050, vec![frame(-1, vec![[-12.0, -12.0, 12.0, 12.0]], vec![])]));
    d
}

fn dragon(m: &Match) -> usize {
    (0..m.fighters.len()).find(|&i| m.fighters[i].is_helper && m.fighters[i].helper_id == 3050).expect("dragon helper")
}

fn fire(m: &mut Match) -> usize {
    enter(m, 0, 3000);
    ticks(m, 2);
    dragon(m)
}

#[test]
fn dragon_climbs_to_an_airborne_enemy() {
    let mut m = duel(dragon_char("goku"), mugen_char("foe", ""));
    face_off(&mut m, 160.0);
    m.fighters[1].pos.y = -110.0;
    let d = fire(&mut m);
    ticks(&mut m, 25);
    assert!(m.fighters[1].life < m.fighters[1].life_max, "dragon reaches the raised target");
    assert!(m.fighters[d].pos.y < -90.0, "it climbed to get there");
}

#[test]
fn dragon_turns_around_for_an_enemy_behind() {
    let mut m = duel(dragon_char("goku"), mugen_char("foe", ""));
    face_off(&mut m, 160.0);
    m.fighters[0].facing = -1.0;
    let d = fire(&mut m);
    ticks(&mut m, 40);
    assert!(m.fighters[1].life < m.fighters[1].life_max, "dragon reverses and hits the foe behind its owner");
    assert!(m.fighters[d].pos.x > m.fighters[0].pos.x, "it flew back across the owner toward the foe");
}

#[test]
fn dragon_picks_the_nearest_living_foe_in_a_team_fight() {
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(dragon_char("goku"), 0, 0, 1);
    let far = m.add_player(mugen_char("far", ""), 1, 0, 1);
    let near = m.add_player(mugen_char("near", ""), 1, 0, 1);
    let fallen = m.add_player(mugen_char("fallen", ""), 1, 0, 1);
    m.start_round();
    while m.round_state != 2 {
        m.tick(&[0, 0, 0, 0]);
    }
    let c = m.camera.x;
    m.fighters[0].pos.x = c - 150.0;
    m.fighters[far].pos.x = c + 140.0;
    m.fighters[near].pos.x = c - 40.0;
    m.fighters[near].pos.y = -60.0;
    m.fighters[fallen].pos.x = c - 130.0;
    m.fighters[fallen].alive = false;
    enter(&mut m, 0, 3000);
    for _ in 0..30 {
        m.tick(&[0, 0, 0, 0]);
    }
    assert!(m.fighters[near].life < m.fighters[near].life_max, "the near living foe is hit");
    assert_eq!(m.fighters[far].life, m.fighters[far].life_max, "the far foe is ignored");
    assert_eq!(m.fighters[fallen].life, m.fighters[fallen].life_max, "a fallen foe is not chosen");
    assert_eq!(m.nearest_foe(0), Some(near));
}

