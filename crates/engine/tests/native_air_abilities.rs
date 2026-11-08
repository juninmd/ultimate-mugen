mod support;
use std::sync::Arc;
use support::*;
use ume_engine::chardef::CharDef;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::matchs::Match;
use ume_engine::native::{compile_native, NativeChar};

const AIR_STATE_OFFSET: i32 = 20000;
const GAPS: [f32; 8] = [24.0, 44.0, 64.0, 90.0, 120.0, 160.0, 220.0, 300.0];
const HEIGHTS: [f32; 3] = [-20.0, -45.0, -70.0];
const WATCH_TICKS: usize = 260;

fn roster() -> Vec<(NativeChar, Arc<CharDef>)> {
    let mut chars: Vec<NativeChar> = std::fs::read_dir("../../assets/chars")
        .unwrap()
        .filter_map(|entry| {
            let path = entry.unwrap().path();
            let stem = path.file_stem()?.to_string_lossy().to_string();
            (path.extension()? == "json" && stem != "index")
                .then(|| serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap())
        })
        .collect();
    chars.sort_by(|a, b| a.id.cmp(&b.id));
    chars
        .into_iter()
        .map(|nc| {
            let def = Arc::new(compile_native(&nc, 1, true));
            (nc, def)
        })
        .collect()
}

fn arena(def: &Arc<CharDef>) -> Match {
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(def.clone(), 0, 0, 1);
    m.add_player(def.clone(), 1, 0, 2);
    m.start_round();
    while m.round_state != 2 {
        m.tick(&[0, 0]);
    }
    m
}

struct Outcome {
    connected: bool,
    landed_in_control: bool,
    finite: bool,
}

fn perform(base: &Match, state: i32) -> Outcome {
    let mut outcome = Outcome {
        connected: false,
        landed_in_control: true,
        finite: true,
    };
    for gap in GAPS {
        for height in HEIGHTS {
            let mut m = base.clone();
            face_off(&mut m, gap);
            m.fighters[0].power = 3000.0;
            m.fighters[0].pos.y = height;
            let life = m.fighters[1].life;
            enter(&mut m, 0, state);
            for _ in 0..WATCH_TICKS {
                m.tick(&[0, 0]);
                let f = &m.fighters[0];
                outcome.finite &= f.pos.x.is_finite()
                    && f.pos.y.is_finite()
                    && f.vel.x.is_finite()
                    && f.vel.y.is_finite();
            }
            outcome.connected |= m.fighters[1].life < life;
            let f = &m.fighters[0];
            outcome.landed_in_control &= f.pos.y >= -0.5 && (f.ctrl || f.move_type == 'H');
        }
    }
    outcome
}

#[test]
fn every_special_and_super_can_be_used_in_the_air_and_returns_to_the_ground() {
    let (mut problems, mut total, mut connecting) = (Vec::new(), 0, 0);
    for (nc, def) in roster() {
        let base = arena(&def);
        let mut air_states: Vec<i32> = def
            .states
            .keys()
            .copied()
            .filter(|state| *state >= AIR_STATE_OFFSET)
            .collect();
        air_states.sort();
        let ground_specials = def
            .states
            .keys()
            .filter(|state| (1000..3800).contains(*state))
            .count();
        println!(
            "{:8} {} air variants for {} ground specials and supers",
            nc.id,
            air_states.len(),
            ground_specials
        );
        assert!(!air_states.is_empty(), "{} has no air variants", nc.id);
        for state in air_states {
            let outcome = perform(&base, state);
            total += 1;
            connecting += outcome.connected as i32;
            if !outcome.finite {
                problems.push(format!(
                    "{} air state {state} produced a non finite value",
                    nc.id
                ));
            }
            if !outcome.landed_in_control {
                problems.push(format!("{} air state {state} did not land and recover control within {WATCH_TICKS} ticks", nc.id));
            }
            if !outcome.connected {
                println!(
                    "  note: {} air state {state} never connected from the tested heights and gaps",
                    nc.id
                );
            }
        }
    }
    println!("{connecting}/{total} air variants connect");
    assert!(
        problems.is_empty(),
        "air abilities broken:\n{}",
        problems.join("\n")
    );
    assert_eq!(
        connecting, total,
        "every air variant must connect from some gap"
    );
}
