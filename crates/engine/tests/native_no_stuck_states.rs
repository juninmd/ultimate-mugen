use std::sync::Arc;
use ume_engine::ai::Ai;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::input::InputSource;
use ume_engine::matchs::Match;
use ume_engine::native::{compile_native, NativeChar};

const ROUND_TICKS: usize = 60 * 40;
const LONGEST_HIT_STATE: i32 = 150;
const LYING_STATES: [i32; 4] = [5110, 5120, 5150, 5170];

fn roster() -> Vec<(String, Arc<ume_engine::chardef::CharDef>)> {
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
        .iter()
        .map(|nc| (nc.id.clone(), Arc::new(compile_native(nc, 1, true))))
        .collect()
}

#[test]
fn no_fighter_idles_inside_a_hit_state() {
    let roster = roster();
    let mut stuck = Vec::new();
    for i in 0..roster.len() {
        for j in (i + 1)..roster.len() {
            let mut m = Match::new(common_def(), None, flat_stage());
            m.add_player(roster[i].1.clone(), 0, 0, 1);
            m.add_player(roster[j].1.clone(), 1, 0, 2);
            m.start_round();
            let (mut a, mut b) = (Ai::new(6, 7), Ai::new(6, 13));
            let roots = m.root_indices();
            let mut runs = [(0i32, 0i32); 2];
            for _ in 0..ROUND_TICKS {
                let inputs = [a.poll(&m, roots[0]), b.poll(&m, roots[1])];
                m.tick(&inputs);
                if m.round_state != 2 {
                    continue;
                }
                for side in 0..2 {
                    let f = &m.fighters[roots[side]];
                    let in_hit_state = f.alive
                        && f.move_type == 'H'
                        && !LYING_STATES.contains(&f.state_no)
                        && m.pause.is_none();
                    runs[side] = if in_hit_state && runs[side].0 == f.state_no {
                        (f.state_no, runs[side].1 + 1)
                    } else if in_hit_state {
                        (f.state_no, 1)
                    } else {
                        (0, 0)
                    };
                    if runs[side].1 == LONGEST_HIT_STATE + 1 {
                        stuck.push(format!(
                            "{} (vs {}) stayed in hit state {} for {} ticks",
                            roster[[i, j][side]].0,
                            roster[[j, i][side]].0,
                            f.state_no,
                            LONGEST_HIT_STATE
                        ));
                    }
                }
            }
        }
    }
    assert!(
        stuck.is_empty(),
        "fighters stuck in a hit state:\n{}",
        stuck.join("\n")
    );
}
