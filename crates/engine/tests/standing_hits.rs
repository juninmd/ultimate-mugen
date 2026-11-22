use std::sync::Arc;
use ume_engine::chardef::{CharDef, Param, Source, StateDef};
use ume_engine::fighter::HitDefData;
use ume_engine::matchs::{KnockdownRule, KnockdownScope, Match, StageInfo};

fn state(id: i32, ty: &str, mv: &str, phys: &str) -> StateDef {
    StateDef { id, params: vec![("type".into(), Param::new(ty)), ("movetype".into(), Param::new(mv)), ("physics".into(), Param::new(phys))], controllers: vec![] }
}

fn dummy(source: Source) -> Arc<CharDef> {
    let mut cd = CharDef::empty("dummy");
    cd.source = source;
    cd.local_w = 320.0;
    cd.local_h = 240.0;
    for s in [state(0, "S", "I", "S"), state(5000, "S", "H", "S"), state(5020, "A", "H", "N"), state(5030, "A", "H", "N"), state(5110, "L", "H", "N")] {
        cd.states.insert(s.id, s);
    }
    Arc::new(cd)
}

fn duel(attacker: Source, rule: KnockdownRule) -> (Match, usize, usize) {
    let mut m = Match::new(dummy(attacker), None, StageInfo::default());
    m.knockdown = rule;
    let a = m.add_player(dummy(attacker), 0, 0, 1);
    let b = m.add_player(dummy(Source::Native), 1, 0, 2);
    m.round_state = 2;
    (m, a, b)
}

fn launcher(damage_percent_of_life: f32, life: f32) -> HitDefData {
    HitDefData {
        damage: life * damage_percent_of_life / 100.0,
        p2stateno: -1,
        hitflag: "MAF".into(),
        attr_state: "S".into(),
        attr_kind: vec!["NA".into()],
        fall: true,
        ground_vel: [-5.0, -7.0],
        airjuggle: 15,
        ..Default::default()
    }
}

fn native_rule() -> KnockdownRule {
    KnockdownRule { scope: KnockdownScope::Native, threshold_percent: 16.0 }
}

#[test]
fn a_light_launcher_pushes_the_defender_back_but_keeps_them_standing() {
    let (mut m, a, b) = duel(Source::Native, native_rule());
    let life = m.fighters[b].life_max;
    m.apply_hit(a, b, &launcher(5.0, life), (0.0, 0.0), None);
    let f = &m.fighters[b];
    assert!(!f.ghv.fall, "5% of life must not knock down");
    assert_eq!(f.ghv.yvel, 0.0, "a standing hit has no lift");
    assert!(f.ghv.xvel.abs() > 5.0, "the push is stronger than the authored velocity: {}", f.ghv.xvel);
    assert_eq!(f.state_no, 5000);
}

#[test]
fn a_strong_hit_still_knocks_down() {
    let (mut m, a, b) = duel(Source::Native, native_rule());
    let life = m.fighters[b].life_max;
    m.apply_hit(a, b, &launcher(20.0, life), (0.0, 0.0), None);
    assert!(m.fighters[b].ghv.fall);
    assert_eq!(m.fighters[b].ghv.yvel, -7.0);
}

#[test]
fn hyper_hits_always_keep_their_explicit_fall() {
    let (mut m, a, b) = duel(Source::Native, native_rule());
    let life = m.fighters[b].life_max;
    let mut hd = launcher(3.0, life);
    hd.attr_kind = vec!["HA".into()];
    m.apply_hit(a, b, &hd, (0.0, 0.0), None);
    assert!(m.fighters[b].ghv.fall);
}

#[test]
fn bounces_and_trips_are_never_softened() {
    for tweak in [|h: &mut HitDefData| h.down_bounce = true, |h: &mut HitDefData| h.wall_bounce = true, |h: &mut HitDefData| h.ground_type = 3] {
        let (mut m, a, b) = duel(Source::Native, native_rule());
        let life = m.fighters[b].life_max;
        let mut hd = launcher(3.0, life);
        tweak(&mut hd);
        m.apply_hit(a, b, &hd, (0.0, 0.0), None);
        assert!(m.fighters[b].ghv.fall);
    }
}

#[test]
fn the_killing_blow_still_falls() {
    let (mut m, a, b) = duel(Source::Native, native_rule());
    let life = m.fighters[b].life_max;
    m.fighters[b].life = 10.0;
    let mut hd = launcher(5.0, life);
    hd.kill = true;
    m.apply_hit(a, b, &hd, (0.0, 0.0), None);
    assert!(m.fighters[b].ghv.fall);
}

#[test]
fn a_long_combo_ends_in_a_knockdown() {
    let (mut m, a, b) = duel(Source::Native, native_rule());
    let life = m.fighters[b].life_max;
    let hd = launcher(10.0, life);
    let falls: Vec<bool> = (0..6)
        .map(|_| {
            m.fighters[b].pos.y = 0.0;
            m.apply_hit(a, b, &hd, (0.0, 0.0), None);
            m.fighters[b].ghv.fall
        })
        .collect();
    assert!(!falls[0] && falls.iter().any(|f| *f), "combo of 10% hits must end in a fall: {falls:?}");
}

#[test]
fn an_airborne_defender_is_untouched_by_the_rule() {
    let (mut m, a, b) = duel(Source::Native, native_rule());
    let life = m.fighters[b].life_max;
    m.fighters[b].pos.y = -50.0;
    m.fighters[b].state_type = 'A';
    m.apply_hit(a, b, &launcher(3.0, life), (0.0, 0.0), None);
    assert_eq!(m.fighters[b].state_no, 5020);
}

#[test]
fn mugen_attackers_keep_their_authored_falls_unless_the_scope_is_everyone() {
    let (mut m, a, b) = duel(Source::Mugen, native_rule());
    let life = m.fighters[b].life_max;
    m.apply_hit(a, b, &launcher(3.0, life), (0.0, 0.0), None);
    assert!(m.fighters[b].ghv.fall, "M.U.G.E.N hitdefs are deliberate");
    let everyone = KnockdownRule { scope: KnockdownScope::All, threshold_percent: 16.0 };
    let (mut m, a, b) = duel(Source::Mugen, everyone);
    m.apply_hit(a, b, &launcher(3.0, life), (0.0, 0.0), None);
    assert!(!m.fighters[b].ghv.fall);
}

#[test]
fn the_classic_setting_changes_nothing() {
    let (mut m, a, b) = duel(Source::Native, KnockdownRule::OFF);
    let life = m.fighters[b].life_max;
    m.apply_hit(a, b, &launcher(3.0, life), (0.0, 0.0), None);
    assert!(m.fighters[b].ghv.fall);
    assert_eq!(m.fighters[b].ghv.yvel, -7.0);
}
