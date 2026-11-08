//! Unit tests for pure combat simulation logic: box collisions, hit detection,
//! state transitions, and variable persistence.

use std::sync::Arc;
use ume_engine::chardef::{AnimFrame, Animation, Box4, CharDef, Controller, Param, StateDef};
use ume_engine::fighter::{Fighter, HitDefData, V2};
use ume_engine::matchs::{Match, StageInfo};

fn make_dummy_chardef(name: &str) -> Arc<CharDef> {
    let mut cd = CharDef::empty(name);
    cd.display_name = name.into();
    cd.local_w = 320.0;
    cd.local_h = 240.0;

    // Add idle animation (0) with hurtbox clsn2 and attackbox clsn1
    let anim0 = Animation {
        id: 0,
        total_time: 10,
        loop_start: 0,
        frames: vec![AnimFrame {
            group: 0,
            index: 0,
            x: 0.0,
            y: 0.0,
            time: 10,
            flip_h: false,
            flip_v: false,
            blend: ume_formats::air::Blend::None,
            alpha: None,
            xscale: 1.0,
            yscale: 1.0,
            angle: 0.0,
            clsn1: vec![[10.0, -50.0, 40.0, -20.0]], // attack box
            clsn2: vec![[-15.0, -70.0, 15.0, 0.0]],  // hurt box
        }],
    };
    cd.anims.insert(0, anim0);

    // Add state 0 (idle)
    let state0 = StateDef {
        id: 0,
        params: vec![
            ("type".into(), Param::new("S")),
            ("movetype".into(), Param::new("I")),
            ("physics".into(), Param::new("S")),
        ],
        controllers: vec![],
    };
    cd.states.insert(0, state0);

    // Add hit state 5000 (stand get hit)
    let state5000 = StateDef {
        id: 5000,
        params: vec![
            ("type".into(), Param::new("S")),
            ("movetype".into(), Param::new("H")),
            ("physics".into(), Param::new("S")),
        ],
        controllers: vec![],
    };
    cd.states.insert(5000, state5000);

    // Add hit state 5010 (crouch get hit)
    let state5010 = StateDef {
        id: 5010,
        params: vec![
            ("type".into(), Param::new("C")),
            ("movetype".into(), Param::new("H")),
            ("physics".into(), Param::new("C")),
        ],
        controllers: vec![],
    };
    cd.states.insert(5010, state5010);

    // Add hit state 5020 (air get hit)
    let state5020 = StateDef {
        id: 5020,
        params: vec![
            ("type".into(), Param::new("A")),
            ("movetype".into(), Param::new("H")),
            ("physics".into(), Param::new("A")),
        ],
        controllers: vec![],
    };
    cd.states.insert(5020, state5020);

    Arc::new(cd)
}

#[test]
fn test_box_overlap_pure_math() {
    let a: Vec<Box4> = vec![[0.0, 0.0, 10.0, 10.0]];
    let b_overlap: Vec<Box4> = vec![[5.0, 5.0, 15.0, 15.0]];
    let b_separate: Vec<Box4> = vec![[11.0, 11.0, 20.0, 20.0]];
    let b_touching: Vec<Box4> = vec![[10.0, 0.0, 20.0, 10.0]];

    // Overlapping boxes should return midpoint of intersection
    let hit = Match::overlap(&a, &b_overlap);
    assert!(hit.is_some());
    let (hx, hy) = hit.unwrap();
    assert_eq!(hx, 7.5);
    assert_eq!(hy, 7.5);

    // Separated boxes should return None
    assert!(Match::overlap(&a, &b_separate).is_none());

    // Touching on edges should count as overlap
    assert!(Match::overlap(&a, &b_touching).is_some());

    // Empty list
    assert!(Match::overlap(&[], &b_overlap).is_none());
}

#[test]
fn test_fighter_boxes_facing_and_scaling() {
    let cd = make_dummy_chardef("hero");
    let mut f = Fighter::new(1, cd.clone(), 0, 1);
    f.anim_real = 0;
    f.elem = 0;
    f.pos = V2 { x: 100.0, y: 200.0 };
    f.facing = 1.0;

    // Facing right (1.0): clsn1 was [10.0, -50.0, 40.0, -20.0]
    let atk_boxes = f.boxes(1);
    assert_eq!(atk_boxes.len(), 1);
    let b = atk_boxes[0];
    assert_eq!(b[0], 110.0); // 100 + 10
    assert_eq!(b[1], 150.0); // 200 + (-50)
    assert_eq!(b[2], 140.0); // 100 + 40
    assert_eq!(b[3], 180.0); // 200 + (-20)

    // Facing left (-1.0): horizontal axis flips around pos.x
    f.facing = -1.0;
    let flipped_boxes = f.boxes(1);
    assert_eq!(flipped_boxes.len(), 1);
    let fb = flipped_boxes[0];
    assert_eq!(fb[0], 60.0); // 100 - 40
    assert_eq!(fb[2], 90.0); // 100 - 10
    assert_eq!(fb[1], 150.0); // y doesn't flip horizontally
    assert_eq!(fb[3], 180.0);
}

#[test]
fn test_match_state_change_and_vars() {
    let cd = make_dummy_chardef("p1");
    let common = cd.clone();
    let mut m = Match::new(common, None, StageInfo::default());

    let p1_idx = m.add_player(cd.clone(), 0, 0, 1);
    let p2_idx = m.add_player(cd.clone(), 1, 0, 2);

    assert_eq!(m.fighters.len(), 2);
    assert_eq!(m.fighters[p1_idx].vars[0], 0);

    // Apply varset controller
    let mut ctrl = Controller {
        kind: "varset".into(),
        label: "VarSet".into(),
        trigger_all: vec![],
        triggers: vec![],
        params: vec![
            ("v".into(), Param::new("0")),
            ("value".into(), Param::new("42")),
        ],
        persistent: 1,
        ignore_hitpause: false,
        uid: 100,
    };
    ume_engine::controllers::run(&mut m, p1_idx, &ctrl);
    assert_eq!(m.fighters[p1_idx].vars[0], 42);

    // Apply varadd controller
    ctrl.kind = "varadd".into();
    ctrl.params = vec![
        ("v".into(), Param::new("0")),
        ("value".into(), Param::new("8")),
    ];
    ume_engine::controllers::run(&mut m, p1_idx, &ctrl);
    assert_eq!(m.fighters[p1_idx].vars[0], 50);

    // Test fvar
    let fctrl = Controller {
        kind: "varset".into(),
        label: "FVarSet".into(),
        trigger_all: vec![],
        triggers: vec![],
        params: vec![
            ("fv".into(), Param::new("3")),
            ("value".into(), Param::new("1.75")),
        ],
        persistent: 1,
        ignore_hitpause: false,
        uid: 101,
    };
    ume_engine::controllers::run(&mut m, p1_idx, &fctrl);
    assert_eq!(m.fighters[p1_idx].fvars[3], 1.75);

    // Test change_state
    m.change_state(p1_idx, 0, cd.clone());
    assert_eq!(m.fighters[p1_idx].state_no, 0);
    assert_eq!(m.fighters[p1_idx].move_type, 'I');
    assert_eq!(m.fighters[p1_idx].state_type, 'S');

    // Test change_state to hit state 5000
    m.change_state(p2_idx, 5000, cd.clone());
    assert_eq!(m.fighters[p2_idx].state_no, 5000);
    assert_eq!(m.fighters[p2_idx].move_type, 'H');

    // Reset p2 to idle (state 0)
    m.change_state(p2_idx, 0, cd.clone());
    assert_eq!(m.fighters[p2_idx].state_no, 0);
    assert_eq!(m.fighters[p2_idx].move_type, 'I');

    // Damage logic via apply_hit with p2stateno = 5000
    let mut hd = HitDefData::default();
    hd.damage = 100.0;
    hd.animtype = 1;
    hd.ground_type = 1;
    hd.p2stateno = 5000;

    let initial_life = m.fighters[p2_idx].life;
    m.apply_hit(p1_idx, p2_idx, &hd, (0.0, 0.0), None);
    assert_eq!(m.fighters[p2_idx].life, initial_life - 100.0);
    assert_eq!(m.fighters[p2_idx].state_no, 5000);
    assert_eq!(m.fighters[p2_idx].move_type, 'H');
}

#[test]
fn test_hit_filtering_can_be_hit() {
    let cd = make_dummy_chardef("p1");
    let common = cd.clone();
    let mut m = Match::new(common, None, StageInfo::default());

    let p1_idx = m.add_player(cd.clone(), 0, 0, 1);
    let p2_idx = m.add_player(cd.clone(), 1, 0, 2);

    let mut hd = HitDefData::default();
    hd.hitflag = "MAF".into(); // Mid, Air, Fall
    hd.attr_state = "S".into();
    hd.attr_kind = vec!["NA".into()];

    // Round state must be 2 (fight) for hits to register normally
    m.round_state = 2;
    assert!(m.can_be_hit(p2_idx, &hd, m.fighters[p1_idx].side));

    // Same side cannot be hit
    assert!(!m.can_be_hit(p1_idx, &hd, m.fighters[p1_idx].side));

    // not_hit_by immunity
    m.fighters[p2_idx].not_hit_by.push(("S,NA".into(), 10));
    assert!(!m.can_be_hit(p2_idx, &hd, m.fighters[p1_idx].side));
}

fn eval_trigger(m: &Match, i: usize, src: &str) -> f64 {
    m.ev(i, &ume_engine::chardef::safe_expr(src))
}

#[test]
fn test_uniq_hit_count_diverges_from_hit_count_on_repeated_hitdef() {
    let cd = make_dummy_chardef("p1");
    let mut m = Match::new(cd.clone(), None, StageInfo::default());
    let p1 = m.add_player(cd.clone(), 0, 0, 1);
    let p2 = m.add_player(cd.clone(), 1, 0, 2);
    m.round_state = 2;

    let mut hd = HitDefData::default();
    hd.damage = 10.0;
    hd.id = 7;
    hd.p2stateno = 5000;
    hd.num_hits = 1;
    hd.p1stateno = -1;
    for _ in 0..3 {
        m.apply_hit(p1, p2, &hd, (0.0, 0.0), None);
    }
    assert_eq!(eval_trigger(&m, p1, "HitCount"), 3.0);
    assert_eq!(eval_trigger(&m, p1, "UniqHitCount"), 1.0);

    let mut other = hd.clone();
    other.id = 8;
    m.apply_hit(p1, p2, &other, (0.0, 0.0), None);
    assert_eq!(eval_trigger(&m, p1, "HitCount"), 4.0);
    assert_eq!(eval_trigger(&m, p1, "UniqHitCount"), 2.0);
}

#[test]
fn test_uniq_hit_count_resets_with_hit_count() {
    let cd = make_dummy_chardef("p1");
    let mut m = Match::new(cd.clone(), None, StageInfo::default());
    let p1 = m.add_player(cd.clone(), 0, 0, 1);
    let p2 = m.add_player(cd.clone(), 1, 0, 2);
    m.round_state = 2;

    let mut hd = HitDefData::default();
    hd.p2stateno = 5000;
    hd.num_hits = 1;
    hd.p1stateno = -1;
    m.apply_hit(p1, p2, &hd, (0.0, 0.0), None);
    m.apply_hit(p1, p2, &hd, (0.0, 0.0), None);
    assert_eq!(eval_trigger(&m, p1, "UniqHitCount"), 1.0);

    m.change_state(p1, 0, cd.clone());
    assert_eq!(eval_trigger(&m, p1, "HitCount"), 0.0);
    assert_eq!(eval_trigger(&m, p1, "UniqHitCount"), 0.0);
    m.apply_hit(p1, p2, &hd, (0.0, 0.0), None);
    assert_eq!(eval_trigger(&m, p1, "UniqHitCount"), 1.0);
}
