mod support;
use std::collections::HashMap;
use support::*;
use ume_engine::chardef::safe_expr;
use ume_engine::cns::parse_states;
use ume_engine::fighter::HitDefData;
use ume_engine::flags::{asf, gsf, lookup, unknown_in_states, FlagTarget};
use ume_engine::matchs::{Announce, AnnounceKind, Match};

const IKEMEN_FLAGS: [&str; 77] = [
    "invisible",
    "noairguard",
    "noautoturn",
    "nocrouchguard",
    "nojugglecheck",
    "noko",
    "noshadow",
    "nostandguard",
    "nowalk",
    "unguardable",
    "animatehitpause",
    "animfreeze",
    "autoguard",
    "drawunder",
    "noaibuttonjam",
    "noaicheat",
    "noailevel",
    "noairjump",
    "nobrake",
    "nocombodisplay",
    "nocornerpush",
    "nocrouch",
    "nodizzypointsdamage",
    "nofacedisplay",
    "nofacep2",
    "nofallcount",
    "nofalldefenceup",
    "nofallhitflag",
    "nofastrecoverfromliedown",
    "nogetupfromliedown",
    "noguardbardisplay",
    "noguarddamage",
    "noguardko",
    "noguardpointsdamage",
    "nohardcodedkeys",
    "nohitdamage",
    "noinput",
    "nointroreset",
    "nojump",
    "nokofall",
    "nokovelocity",
    "nolifebaraction",
    "nolifebardisplay",
    "nomakedust",
    "nonamedisplay",
    "nopowerbardisplay",
    "noredlifedamage",
    "noscore",
    "nostand",
    "nostunbardisplay",
    "noturntarget",
    "nowinicondisplay",
    "postroundinput",
    "projtypecollision",
    "runfirst",
    "runlast",
    "sizepushonly",
    "nodestroyself",
    "globalnoko",
    "globalnoshadow",
    "intro",
    "nobardisplay",
    "nobg",
    "nofg",
    "nokoslow",
    "nokosnd",
    "nomusic",
    "roundnotover",
    "timerfreeze",
    "camerafreeze",
    "notimedisplay",
    "roundfreeze",
    "roundnotskip",
    "skipfightdisplay",
    "skipkodisplay",
    "skiprounddisplay",
    "skipwindisplay",
];

fn asserting(flags: &[&str]) -> String {
    let mut cns =
        String::from("[Statedef 0]\ntype = S\nmovetype = I\nphysics = S\nanim = 0\nctrl = 1\n");
    for (k, flag) in flags.iter().enumerate() {
        cns.push_str(&format!(
            "\n[State 0, f{k}]\ntype = AssertSpecial\ntrigger1 = 1\nflag = {flag}\n"
        ));
    }
    cns
}

#[test]
fn every_ikemen_flag_has_a_bit_and_names_are_unique() {
    let mut char_bits = 0u64;
    let mut global_bits = 0u32;
    for name in IKEMEN_FLAGS {
        match lookup(name).unwrap_or_else(|| panic!("flag {name} missing")) {
            FlagTarget::Character(bit) => {
                assert_eq!(char_bits & bit, 0, "{name} shares a bit");
                char_bits |= bit;
            }
            FlagTarget::Global(bit) => {
                assert_eq!(global_bits & bit, 0, "{name} shares a bit");
                global_bits |= bit;
            }
        }
    }
    assert_eq!(char_bits.count_ones(), 58);
    assert_eq!(global_bits.count_ones(), 19);
    assert!(matches!(
        lookup("immovable"),
        Some(FlagTarget::Character(_))
    ));
    assert_eq!(lookup(" \"NoWalk\" "), lookup("nowalk"));
    assert_eq!(lookup("notaflag"), None);
}

#[test]
fn unknown_flags_are_reported_once_per_name() {
    let mut states = HashMap::new();
    parse_states(
        "[Statedef 1]\n[State 1, a]\ntype = AssertSpecial\ntrigger1 = 1\nflag = NoWalk\nflag2 = Bogus\nflag3 = bogus\n[State 1, b]\ntype = AssertSpecial\ntrigger1 = 1\nflag = Another\n",
        &mut states,
        true,
    );
    assert_eq!(
        unknown_in_states(&states),
        vec!["another".to_string(), "bogus".to_string()]
    );
}

#[test]
fn is_asserted_reads_character_and_global_flags() {
    let mut m = duel(
        mugen_char(
            "p1",
            &asserting(&["NoWalk", "TimerFreeze", "NoMakeDust", "Immovable"]),
        ),
        mugen_char("p2", ""),
    );
    ticks(&mut m, 2);
    let eval = |m: &Match, src: &str| m.ev(0, &safe_expr(src));
    assert_eq!(eval(&m, "IsAsserted(NoWalk)"), 1.0);
    assert_eq!(eval(&m, "IsAsserted(nomakedust)"), 1.0);
    assert_eq!(eval(&m, "IsAsserted(TimerFreeze)"), 1.0);
    assert_eq!(eval(&m, "IsAsserted(NoJump)"), 0.0);
    assert_eq!(eval(&m, "IsAsserted(Intro)"), 0.0);
    assert_eq!(eval(&m, "IsAsserted(Nonsense)"), 0.0);
    assert_eq!(m.ev(1, &safe_expr("IsAsserted(NoWalk)")), 0.0);
}

#[test]
fn assertspecial_enabled_zero_asserts_nothing() {
    let cns = "[Statedef 0]\ntype = S\nmovetype = I\nphysics = S\nanim = 0\nctrl = 1\n[State 0, f]\ntype = AssertSpecial\ntrigger1 = 1\nflag = NoWalk\nenabled = 0\n";
    let mut m = duel(mugen_char("p1", cns), mugen_char("p2", ""));
    ticks(&mut m, 2);
    assert!(!m.fighters[0].has_assert(asf::NOWALK));
}

#[test]
fn character_flags_clear_when_no_longer_asserted() {
    let cns = "[Statedef 0]\ntype = S\nmovetype = I\nphysics = S\nanim = 0\nctrl = 1\n[State 0, f]\ntype = AssertSpecial\ntrigger1 = Time < 3\nflag = NoWalk\n";
    let mut m = duel(mugen_char("p1", cns), mugen_char("p2", ""));
    enter(&mut m, 0, 0);
    ticks(&mut m, 1);
    assert!(m.fighters[0].has_assert(asf::NOWALK));
    ticks(&mut m, 6);
    assert!(!m.fighters[0].has_assert(asf::NOWALK));
}

#[test]
fn noinput_blanks_the_input_history() {
    let mut m = duel(
        mugen_char("p1", &asserting(&["NoInput"])),
        mugen_char("p2", ""),
    );
    ticks(&mut m, 2);
    m.tick(&[ume_engine::input::k::A, 0]);
    m.tick(&[ume_engine::input::k::A, 0]);
    let history = &m.fighters[0].history;
    assert_eq!(history.at(history.tick) & ume_engine::input::k::A, 0);
}

#[test]
fn nofallhitflag_makes_a_falling_target_ignore_the_f_flag() {
    let mut m = duel(
        mugen_char("a", ""),
        mugen_char("b", &asserting(&["NoFallHitFlag"])),
    );
    enter(&mut m, 1, 5000);
    ticks(&mut m, 1);
    m.fighters[1].move_type = 'H';
    m.fighters[1].state_type = 'A';
    m.fighters[1].ghv.fall = true;
    let hit = HitDefData {
        hitflag: "MAFD".into(),
        attr_state: "S".into(),
        airjuggle: 100,
        ..Default::default()
    };
    assert!(!m.can_be_hit(1, &hit, 0));
    m.fighters[1].asserts = 0;
    assert!(m.can_be_hit(1, &hit, 0));
}

#[test]
fn guard_direction_flags_block_guarding() {
    let m = duel(mugen_char("a", ""), mugen_char("b", ""));
    let mut m = m;
    let hit = HitDefData {
        guardflag: "MA".into(),
        hitflag: "MAFD".into(),
        attr_state: "S".into(),
        ..Default::default()
    };
    enter(&mut m, 1, 150);
    ticks(&mut m, 1);
    m.fighters[1].state_type = 'S';
    let events_before = m.events.len();
    m.apply_hit(0, 1, &hit, (0.0, 0.0), None);
    assert!(m.fighters[1].ghv.guarded, "baseline: guard state guards");
    let mut m = duel(mugen_char("a", ""), mugen_char("b", ""));
    enter(&mut m, 1, 150);
    ticks(&mut m, 1);
    m.fighters[1].state_type = 'S';
    m.fighters[1].asserts |= asf::NOSTANDGUARD;
    m.apply_hit(0, 1, &hit, (0.0, 0.0), None);
    assert!(!m.fighters[1].ghv.guarded);
    let _ = events_before;
}

#[test]
fn nohitdamage_and_noko_keep_the_target_alive_at_zero_life() {
    let hit = HitDefData {
        damage: 5000.0,
        kill: true,
        hitflag: "MAFD".into(),
        attr_state: "S".into(),
        ..Default::default()
    };
    let mut m = duel(mugen_char("a", ""), mugen_char("b", ""));
    m.fighters[1].asserts |= asf::NOHITDAMAGE;
    m.apply_hit(0, 1, &hit, (0.0, 0.0), None);
    assert_eq!(m.fighters[1].life, m.fighters[1].life_max);

    let mut m = duel(mugen_char("a", ""), mugen_char("b", ""));
    m.fighters[1].asserts |= asf::NOKO;
    m.apply_hit(0, 1, &hit, (0.0, 0.0), None);
    assert_eq!(m.fighters[1].life, 0.0);
    assert!(m.fighters[1].alive);

    let mut m = duel(mugen_char("a", ""), mugen_char("b", ""));
    m.apply_hit(0, 1, &hit, (0.0, 0.0), None);
    assert!(!m.fighters[1].alive);

    let mut m = duel(mugen_char("a", ""), mugen_char("b", ""));
    m.global_asserts |= gsf::GLOBALNOKO;
    m.apply_hit(0, 1, &hit, (0.0, 0.0), None);
    assert!(m.fighters[1].alive);
}

#[test]
fn nokovelocity_keeps_the_hitdef_velocity_on_ko() {
    let hit = HitDefData {
        damage: 5000.0,
        kill: true,
        hitflag: "MAFD".into(),
        attr_state: "S".into(),
        ..Default::default()
    };
    let mut m = duel(mugen_char("a", ""), mugen_char("b", ""));
    m.apply_hit(0, 1, &hit, (0.0, 0.0), None);
    assert!(m.fighters[1].ghv.yvel < 0.0);
    let mut m = duel(mugen_char("a", ""), mugen_char("b", ""));
    m.fighters[1].asserts |= asf::NOKOVELOCITY;
    m.apply_hit(0, 1, &hit, (0.0, 0.0), None);
    assert_eq!(m.fighters[1].ghv.yvel, 0.0);
}

#[test]
fn immovable_fighter_is_not_pushed_by_the_opponent() {
    let mut m = duel(
        mugen_char("a", &asserting(&["Immovable"])),
        mugen_char("b", ""),
    );
    face_off(&mut m, 4.0);
    let before = m.fighters[0].pos.x;
    ticks(&mut m, 3);
    assert!(
        (m.fighters[0].pos.x - before).abs() < 0.01,
        "immovable moved by {}",
        m.fighters[0].pos.x - before
    );
    assert!(m.fighters[1].pos.x - m.fighters[0].pos.x > 4.0);
}

#[test]
fn animatehitpause_keeps_the_animation_running_during_hitpause() {
    let advance_during_hitpause = |flag: &str| {
        let cns = format!("[Statedef 0]\ntype = S\nmovetype = I\nphysics = S\nanim = 300\nctrl = 1\n[State 0, f]\ntype = AssertSpecial\ntrigger1 = 1\nignorehitpause = 1\nflag = {flag}\n");
        let mut m = duel(mugen_char("a", &cns), mugen_char("b", ""));
        m.fighters[0].hit_pause = 20;
        ticks(&mut m, 2);
        let before = m.fighters[0].anim_elapsed;
        ticks(&mut m, 4);
        m.fighters[0].anim_elapsed - before
    };
    assert_eq!(advance_during_hitpause("NoJump"), 0);
    assert_eq!(advance_during_hitpause("AnimateHitPause"), 4);
}

#[test]
fn animfreeze_stops_the_animation_without_a_pause() {
    let cns = "[Statedef 0]\ntype = S\nmovetype = I\nphysics = S\nanim = 300\nctrl = 1\n[State 0, f]\ntype = AssertSpecial\ntrigger1 = 1\nflag = AnimFreeze\n";
    let mut m = duel(mugen_char("a", cns), mugen_char("b", ""));
    ticks(&mut m, 2);
    let before = m.fighters[0].anim_elapsed;
    ticks(&mut m, 4);
    assert_eq!(m.fighters[0].anim_elapsed, before);
}

const HELPER_PARENT: &str = "
[Statedef 800]
type = S
physics = S

[State 800, h]
type = Helper
trigger1 = Time = 0
id = 5
stateno = 810

[Statedef 810]
type = S
physics = N
anim = 0

[State 810, protect]
type = AssertSpecial
trigger1 = NumHelper(5) > 0 && Time >= 0
flag = NoDestroySelf

[State 810, d]
type = DestroySelf
trigger1 = Time = 3
";

#[test]
fn nodestroyself_prevents_helper_destruction() {
    let mut m = duel(mugen_char("a", HELPER_PARENT), mugen_char("b", ""));
    enter(&mut m, 0, 800);
    ticks(&mut m, 8);
    assert!(m.fighters.iter().any(|f| f.is_helper && !f.destroyed));
    let unprotected = HELPER_PARENT.replace("flag = NoDestroySelf", "flag = NoJump");
    let mut m = duel(mugen_char("a", &unprotected), mugen_char("b", ""));
    enter(&mut m, 0, 800);
    ticks(&mut m, 8);
    assert!(!m.fighters.iter().any(|f| f.is_helper && !f.destroyed));
}

#[test]
fn global_display_flags_skip_announcements() {
    let mut m = duel(
        mugen_char("a", &asserting(&["SkipRoundDisplay", "SkipKoDisplay"])),
        mugen_char("b", ""),
    );
    m.announce = Some(Announce {
        text: "ROUND 1".into(),
        time: 50,
        total: 50,
        kind: AnnounceKind::Round,
    });
    m.tick(&[0, 0]);
    assert!(m.announce.is_none());
    m.announce = Some(Announce {
        text: "FIGHT!".into(),
        time: 50,
        total: 50,
        kind: AnnounceKind::Fight,
    });
    m.tick(&[0, 0]);
    assert!(
        m.announce.is_some(),
        "FIGHT is not skipped by the round/ko flags"
    );
}

#[test]
fn camerafreeze_and_roundfreeze_hold_the_world() {
    let cns = "[Statedef 900]\ntype = S\nmovetype = I\nphysics = S\nanim = 0\nctrl = 1\n[State 900, f]\ntype = AssertSpecial\ntrigger1 = 1\nflag = CameraFreeze\nflag2 = RoundFreeze\n";
    let mut m = duel(mugen_char("a", cns), mugen_char("b", ""));
    enter(&mut m, 0, 900);
    ticks(&mut m, 2);
    let (camera, round_time, timer) = (m.camera.x, m.round_time, m.timer);
    m.fighters[1].pos.x += 120.0;
    ticks(&mut m, 5);
    assert_eq!(m.camera.x, camera);
    assert_eq!(m.round_time, round_time);
    assert_eq!(m.timer, timer);
}

#[test]
fn nocornerpush_stops_the_attacker_being_pushed_back() {
    let hit = HitDefData {
        damage: 10.0,
        hitflag: "MAFD".into(),
        attr_state: "S".into(),
        ground_corner_push: -3.0,
        ..Default::default()
    };
    let run = |flag: bool| {
        let mut m = duel(mugen_char("a", ""), mugen_char("b", ""));
        let right = m.camera.x + m.view_w / 2.0;
        m.fighters[1].pos.x = right;
        m.fighters[0].pos.x = right - 30.0;
        if flag {
            m.fighters[0].asserts |= asf::NOCORNERPUSH;
        }
        m.apply_hit(0, 1, &hit, (0.0, 0.0), None);
        m.fighters[0].vel.x
    };
    assert_eq!(run(false), -3.0);
    assert_eq!(run(true), 0.0);
}

#[test]
fn runfirst_and_runlast_reorder_execution() {
    let mut m = duel(
        mugen_char("a", &asserting(&["RunLast"])),
        mugen_char("b", &asserting(&["RunFirst"])),
    );
    ticks(&mut m, 3);
    assert!(m.fighters[0].has_assert(asf::RUNLAST));
    assert!(m.fighters[1].has_assert(asf::RUNFIRST));
    assert_eq!(m.run_order_ids(), vec![m.fighters[1].id, m.fighters[0].id]);
}
