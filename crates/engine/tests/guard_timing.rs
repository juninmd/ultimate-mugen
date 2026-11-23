use crate::support;
use std::sync::Arc;
use support::*;
use ume_engine::ai::Ai;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::fighter::HitDefData;
use ume_engine::input::{k, InputSource};
use ume_engine::matchs::Match;
use ume_native::{compile_native, NativeChar};

fn kaito_match() -> Match {
    let nc: NativeChar =
        serde_json::from_str(&std::fs::read_to_string("../../assets/chars/kaito.json").unwrap())
            .unwrap();
    let def = Arc::new(compile_native(&nc, 1, true));
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(def.clone(), 0, 0, 1);
    m.add_player(def, 1, 0, 2);
    m.start_round();
    while m.round_state != 2 {
        m.tick(&[0, 0]);
    }
    face_off(&mut m, 60.0);
    m
}

#[test]
fn guarded_hit_reduces_blockstun_and_shake_threefold() {
    let mut m = kaito_match();
    for _ in 0..20 {
        m.tick(&[0, k::R]);
    }
    let mut hd = HitDefData::default();
    hd.guardflag = "MA".into();
    hd.guard_pause_p2 = 9;
    hd.guard_hit_time = 15;
    hd.guard_slide_time = 12;
    hd.guard_ctrl_time = 12;
    m.apply_hit(0, 1, &hd, (0.0, 0.0), None);
    let d = &m.fighters[1];
    assert!(d.ghv.guarded);
    assert_eq!(d.hit_shake, 3);
    assert_eq!(d.hit_over_time, 5);
    assert_eq!(d.slide_time, 4);
    assert_eq!(d.ghv.hittime, 5);
    assert_eq!(d.ghv.slidetime, 4);
    assert_eq!(d.ghv.ctrltime, 4);
}

#[test]
fn air_guard_blocks_a_jump_in_with_guardflag_a() {
    let mut m = kaito_match();
    m.fighters[1].state_type = 'A';
    m.fighters[1].state_no = 50;
    m.fighters[1].ctrl = true;
    m.fighters[1].guard_button = true;
    let mut hd = HitDefData::default();
    hd.guardflag = "MA".into();
    m.apply_hit(0, 1, &hd, (0.0, 0.0), None);
    assert!(m.fighters[1].ghv.guarded, "air guard blocks a jump-in");
}

#[test]
fn a_jumping_fighter_enters_air_guard_when_the_enemy_is_in_range() {
    let mut m = kaito_match();
    m.fighters[1].state_type = 'A';
    m.fighters[1].state_no = 50;
    m.fighters[1].pos.y = -80.0;
    m.fighters[1].ctrl = true;
    m.fighters[1].guard_button = true;
    m.fighters[0].move_type = 'A';
    m.fighters[0].hitdef_active = true;
    let mut hd = HitDefData::default();
    hd.guardflag = "MA".into();
    hd.guard_dist = 200.0;
    m.fighters[0].hitdef = Some(hd);
    m.tick(&[0, k::R]);
    assert!(
        (120..=132).contains(&m.fighters[1].state_no),
        "air guard state {}",
        m.fighters[1].state_no
    );
}

#[test]
fn perfect_guard_reduces_the_scaled_blockstun_further() {
    let mut m = kaito_match();
    m.tick(&[0, k::R]);
    let mut hd = HitDefData::default();
    hd.guardflag = "MA".into();
    hd.guard_pause_p2 = 9;
    hd.guard_hit_time = 15;
    m.apply_hit(0, 1, &hd, (0.0, 0.0), None);
    let d = &m.fighters[1];
    assert!(d.ghv.guarded);
    assert_eq!(d.hit_over_time, 3);
    assert_eq!(d.ghv.hittime, 3);
}

#[test]
fn ai_holds_guard_for_a_third_of_the_previous_duration() {
    let mut saw_guard = false;
    for seed in 1..20u32 {
        let mut m = kaito_match();
        m.fighters[0].move_type = 'A';
        m.fighters[0].state_time = 20;
        m.fighters[1].ctrl = false;
        let mut ai = Ai::new(8, seed);
        let mut guarded_ticks = 0;
        for _ in 0..12 {
            let bits = ai.poll(&m, 1);
            if bits & k::R != 0 {
                guarded_ticks += 1;
                m.fighters[0].move_type = 'I';
            } else if guarded_ticks > 0 {
                break;
            }
        }
        if guarded_ticks > 0 {
            saw_guard = true;
            assert!(
                (4..=6).contains(&guarded_ticks),
                "seed {seed} held guard for {guarded_ticks} ticks"
            );
        }
    }
    assert!(saw_guard);
}

#[test]
fn ai_hits_guard_budget_limit_after_short_sustained_blocking() {
    let mut m = kaito_match();
    m.fighters[0].move_type = 'A';
    m.fighters[0].state_time = 20;
    m.fighters[1].ctrl = false;
    let mut ai = Ai::new(8, 1);
    let mut guarded_ticks = 0;
    for _ in 0..30 {
        if ai.poll(&m, 1) & k::R != 0 {
            guarded_ticks += 1;
        }
    }
    assert!(
        (12..=20).contains(&guarded_ticks),
        "expected budget exhaustion around 16 ticks, saw {guarded_ticks}"
    );
}
