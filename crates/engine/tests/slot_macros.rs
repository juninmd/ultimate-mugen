use ume_engine::commands::input_macro;
use ume_engine::native::{compile_native, NativeChar};

fn chars() -> Vec<(String, NativeChar)> {
    let dir = format!("{}/../../assets/chars", env!("CARGO_MANIFEST_DIR"));
    let mut out = vec![];
    for e in std::fs::read_dir(dir).unwrap().flatten() {
        let p = e.path();
        if p.extension().is_some_and(|x| x == "json") {
            if let Ok(nc) = serde_json::from_str::<NativeChar>(&std::fs::read_to_string(&p).unwrap_or_default()) {
                out.push((p.file_stem().unwrap().to_string_lossy().into_owned(), nc));
            }
        }
    }
    out
}

#[test]
fn every_listed_move_has_a_playable_macro_for_both_facings() {
    let mut broken = vec![];
    for (id, nc) in chars() {
        let d = compile_native(&nc, 1, false);
        for mv in &d.movelist {
            let Some(cmd) = d.commands.iter().find(|c| c.name == mv.command) else {
                broken.push(format!("{id}: {} has no command {}", mv.name, mv.command));
                continue;
            };
            for facing in [1.0, -1.0] {
                if input_macro(cmd, facing).is_empty() {
                    broken.push(format!("{id}: {} ({}) facing {facing}", mv.name, mv.input));
                }
            }
        }
    }
    assert!(broken.is_empty(), "{} unplayable shortcuts:\n{}", broken.len(), broken.join("\n"));
}

#[test]
fn a_shortcut_macro_triggers_the_move() {
    use std::sync::Arc;
    use ume_engine::common::{common_def, flat_stage};
    use ume_engine::matchs::Match;
    let nc = chars().into_iter().find(|(id, _)| id == "kaito").unwrap().1;
    let d = Arc::new(compile_native(&nc, 1, false));
    let dummy = Arc::new(compile_native(&nc, 1, false));
    let mut fired = 0;
    let free: Vec<_> = d.movelist.iter().filter(|m| m.cost == 0.0 && !m.drain).collect();
    assert!(!free.is_empty());
    for mv in free {
        let cmd = d.commands.iter().find(|c| c.name == mv.command).unwrap();
        let mut m = Match::new(common_def(), None, flat_stage());
        m.add_player(d.clone(), 0, 0, 1);
        m.add_player(dummy.clone(), 1, 0, 1);
        m.start_round();
        for _ in 0..130 {
            m.tick(&[0, 0]);
        }
        let facing = m.fighters[0].facing;
        let mut changed = false;
        for b in input_macro(cmd, facing) {
            m.tick(&[b, 0]);
            changed |= m.fighters[0].state_no != 0;
        }
        for _ in 0..4 {
            m.tick(&[0, 0]);
            changed |= m.fighters[0].state_no != 0;
        }
        fired += changed as usize;
    }
    assert!(fired > 0);
}
