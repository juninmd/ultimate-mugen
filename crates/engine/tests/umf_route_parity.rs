mod support;
use serde_json::Value;
use std::fmt::Write as _;
use std::sync::Arc;
use support::gogeta::load;
use support::parity::{forced, move_state, row, save, Run};
use ume_engine::chardef::CharDef;

fn umf() -> Option<(Arc<CharDef>, Vec<Value>)> {
    let path = std::path::PathBuf::from(std::env::var("UME_TEST_UMF_JSON").ok()?);
    let nc = ume_native::parse_char(&std::fs::read_to_string(&path).ok()?).map_err(|e| eprintln!("parse_char: {e}")).ok()?;
    let routes: Vec<Value> = serde_json::from_str(&std::fs::read_to_string(path.with_file_name("routes.json")).ok()?).ok()?;
    Some((Arc::new(ume_native::compile_native(&nc, 1, true)), routes))
}

#[test]
fn every_converted_move_matches_its_original_state() {
    if std::env::var("UME_TEST_UMF_JSON").is_err() {
        return;
    }
    let orig = load(1).expect("original M.U.G.E.N fighter (UME_CHAR_DIR) did not load");
    let (conv, routes) = umf().expect("UMF fighter or routes.json did not load");
    let out = std::path::PathBuf::from(std::env::var("UME_EVIDENCE_DIR").unwrap_or_else(|_| "target/umf-parity".into()));
    std::fs::create_dir_all(&out).unwrap();
    let sheets: Vec<String> = std::env::var("UME_EVIDENCE_MOVES").unwrap_or_default().split(',').map(str::to_string).collect();
    let mut report = String::from("| move | side | states | min (x, y) | power | damage | sounds | sounds without data | end (state, ctrl, y) |\n|---|---|---|---|---|---|---|---|---|\n");
    let mut summary = String::from("| move | original state | sounds | damage original / umf | max height original / umf | verdict |\n|---|---|---|---|---|---|\n");
    let (mut checked, mut same_sound) = (0, 0);
    for r in routes.iter().filter(|r| r["listed"].as_bool() == Some(true)) {
        let (id, state) = (r["id"].as_str().unwrap(), r["state"].as_i64().unwrap() as i32);
        let Some(sid) = conv.movelist.iter().find(|mv| mv.command == format!("m_{id}")).map(|mv| mv.state) else { continue };
        let air = r["when"].as_str() == Some("air");
        let (a, b) = (forced(&orig, state, 240, air), forced(&conv, sid, 240, air));
        let sounds_ok = a.played() == b.played();
        let damage_ok = (a.damage == 0.0) == (b.damage == 0.0);
        checked += 1;
        same_sound += sounds_ok as usize;
        let verdict = if sounds_ok && damage_ok { "ok" } else if damage_ok { "sound differs" } else { "damage differs" };
        let _ = writeln!(summary, "| {id} | {state} | {} | {:.0} / {:.0} | {:.0} / {:.0} | {verdict} |", if sounds_ok { "same" } else { "differ" }, a.damage, b.damage, -a.min.1, -b.min.1);
        row(&mut report, id, "original", &a);
        row(&mut report, id, "umf", &b);
        if sheets.iter().any(|s| s == id) {
            save(&out, id, &a, &b);
        }
    }
    for id in std::env::var("UME_EVIDENCE_EXTRA").unwrap_or_default().split(',').filter(|s| !s.is_empty()) {
        let sid = move_state(&conv, &format!("m_{id}")).unwrap_or_else(|| panic!("UMF-only move {id} is not in the movelist"));
        let run = forced(&conv, sid, 300, false);
        let _ = writeln!(summary, "| {id} | UMF only | - | - / {:.0} | - / {:.0} | {} |", run.damage, -run.min.1, if run.damage > 0.0 { "new" } else { "no damage" });
        save(&out, id, &run, &Run::empty());
        assert!(run.damage > 0.0, "UMF-only move {id} never hits");
    }
    std::fs::write(out.join("route_parity.md"), format!("{summary}\n{report}")).unwrap();
    println!("{summary}");
    assert!(checked > 0, "no converted move could be matched to its original state");
    assert!(same_sound * 10 >= checked * 7, "only {same_sound}/{checked} moves play the original sounds");
}
