mod support;
use std::sync::Arc;
use support::*;
use ume_engine::chardef::CharDef;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::matchs::Match;
use ume_engine::native::{compile_native, NativeChar};

const GAPS: [f32; 8] = [14.0, 24.0, 34.0, 44.0, 54.0, 64.0, 76.0, 90.0];

fn characters() -> Vec<NativeChar> {
    let dir = std::env::var("UME_CHARS_DIR").unwrap_or_else(|_| "../../assets/chars".into());
    let mut out: Vec<NativeChar> = std::fs::read_dir(dir).unwrap().filter_map(|e| {
        let path = e.unwrap().path();
        let stem = path.file_stem()?.to_string_lossy().to_string();
        (path.extension()? == "json" && stem != "index").then(|| serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap())
    }).collect();
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}

fn move_state_and_command(def: &CharDef, name: &str) -> Option<(i32, String)> {
    let state = def.states.get(&-1)?;
    let label = format!("State -1, {name}");
    let controller = state.controllers.iter().find(|c| c.kind == "changestate" && c.label == label)?;
    let sid: i32 = controller.raw("value")?.trim().parse().ok()?;
    let command = def.movelist.iter().find(|m| m.name == name)?.command.clone();
    Some((sid, command))
}

fn arena(def: &Arc<CharDef>) -> Match {
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(def.clone(), 0, 0, 1);
    m.add_player(def.clone(), 1, 0, 2);
    m.start_round();
    while m.round_state != 2 { m.tick(&[0, 0]); }
    m
}

fn cancels_into(base: &Match, from: i32, to: (i32, &str), crouching: bool) -> bool {
    for gap in GAPS {
        let mut m = base.clone();
        face_off(&mut m, gap);
        m.fighters[0].power = 3000.0;
        let before = m.fighters[1].life;
        enter(&mut m, 0, from);
        let mut contact = false;
        for _ in 0..90 {
            m.tick(&[0, 0]);
            if (m.fighters[0].move_contact > 0 || m.fighters[1].life < before) && !contact { contact = true; }
            if contact { m.fighters[0].asserted_commands.push((to.1.to_string(), 2)); if crouching { m.fighters[0].asserted_commands.push(("holddown".to_string(), 2)); } }
            if contact && m.fighters[0].state_no == to.0 { return true; }
        }
        if m.fighters[1].life < before { return false; }
    }
    false
}

#[test]
fn declared_cancel_routes_chain_on_contact() {
    let mut checked = 0;
    let mut failures = vec![];
    for nc in characters() {
        let def = Arc::new(compile_native(&nc, 1, true));
        let base = arena(&def);
        for source in nc.moves.iter().filter(|m| m.cancels.is_some()) {
            let Some((from, _)) = move_state_and_command(&def, &source.name) else { continue };
            for target_id in source.cancels.iter().flatten() {
                let target = nc.moves.iter().find(|m| &m.id == target_id).unwrap();
                let Some((to, command)) = move_state_and_command(&def, &target.name) else { failures.push(format!("{}: no state for {}", nc.id, target.name)); continue };
                checked += 1;
                if !cancels_into(&base, from, (to, &command), target.when.as_deref() == Some("crouch")) { failures.push(format!("{}: {} -> {} did not chain", nc.id, source.id, target.id)); }
            }
        }
    }
    println!("checked {checked} declared cancels");
    assert!(checked >= 30, "roster declares explicit cancel routes, checked {checked}");
    assert!(failures.is_empty(), "broken routes:\n{}", failures.join("\n"));
}
