mod support;
use std::sync::Arc;
use ume_engine::ai::Ai;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::input::{k, InputSource};
use ume_engine::matchs::Match;
use ume_engine::native::{compile_native, NativeChar};

const LEVEL: i32 = 8;
const SEEDS: u32 = 60;
const WATCH_POLLS: usize = 14;

fn arena() -> Match {
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
    support::face_off(&mut m, 90.0);
    m
}

fn performs_dragon_punch(stream: &[u32]) -> bool {
    let forward = k::R;
    let Some(start) = stream.iter().position(|b| b & 15 == forward) else {
        return false;
    };
    let Some(down) = stream[start..]
        .iter()
        .position(|b| b & 15 == k::D)
        .map(|i| i + start)
    else {
        return false;
    };
    let Some(diagonal) = stream[down..]
        .iter()
        .position(|b| b & 15 == (k::D | forward))
        .map(|i| i + down)
    else {
        return false;
    };
    stream[diagonal..].iter().any(|b| b & k::Y != 0)
}

fn share_performing_dragon_punch(setup: impl Fn(&mut Match)) -> f32 {
    let (mut hits, base) = (0, arena());
    for seed in 0..SEEDS {
        let mut m = base.clone();
        setup(&mut m);
        let mut ai = Ai::new(LEVEL, seed * 7919 + 13);
        let stream: Vec<u32> = (0..WATCH_POLLS).map(|_| ai.poll(&m, 0)).collect();
        if performs_dragon_punch(&stream) {
            hits += 1;
        }
    }
    hits as f32 / SEEDS as f32
}

#[test]
fn a_descending_rival_in_range_is_met_with_a_dragon_punch() {
    let airborne = share_performing_dragon_punch(|m| {
        m.fighters[1].state_type = 'A';
        m.fighters[1].vel.y = 6.0;
    });
    let grounded = share_performing_dragon_punch(|_| {});
    assert!(airborne >= 0.4, "anti-air rate {airborne}");
    assert!(
        airborne > grounded + 0.25,
        "airborne {airborne} grounded {grounded}"
    );
}
