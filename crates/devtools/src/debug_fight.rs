use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use ume_engine::ai::Ai;
use ume_engine::chardef::CharDef;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::input::InputSource;
use ume_engine::loader::load_mugen_char;
use ume_engine::matchs::{Event, Match};
use ume_formats::vfs::{add_dir, add_zip, find_char_defs, Vfs};
use ume_native::{compile_native, parse_char};

#[derive(Clone, Debug)]
pub struct DebugOptions {
    pub p1: String,
    pub p2: Option<String>,
    pub frames: usize,
    pub only_moves: bool,
}

pub fn load_fighter(spec: &str) -> Result<Arc<CharDef>, String> {
    let direct = PathBuf::from(spec);
    let candidate = if direct.exists() {
        direct
    } else {
        let root_json = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("assets/chars")
            .join(format!("{spec}.json"));
        let json = PathBuf::from("assets/chars").join(format!("{spec}.json"));
        if json.exists() {
            json
        } else if root_json.exists() {
            root_json
        } else {
            PathBuf::from("mugen/chars").join(spec)
        }
    };
    if candidate.extension().and_then(|e| e.to_str()) == Some("json") {
        let raw = std::fs::read_to_string(&candidate).map_err(|e| format!("{spec}: {e}"))?;
        let nc = parse_char(&raw).map_err(|e| format!("{spec}: {e}"))?;
        return Ok(Arc::new(compile_native(&nc, 1, true)));
    }
    let mut vfs = Vfs::new();
    if candidate.extension().and_then(|e| e.to_str()) == Some("zip") {
        let bytes = std::fs::read(&candidate).map_err(|e| format!("{spec}: {e}"))?;
        add_zip(&mut vfs, &bytes, "")?;
    } else if candidate.is_dir() {
        add_dir(&mut vfs, &candidate, "").map_err(|e| e.to_string())?;
    } else if candidate.is_file() {
        let parent = candidate.parent().unwrap_or_else(|| Path::new("."));
        add_dir(&mut vfs, parent, "").map_err(|e| e.to_string())?;
    } else {
        return Err(format!("fighter not found: {spec}"));
    }
    let defs = find_char_defs(&vfs);
    let chosen = defs
        .first()
        .ok_or_else(|| format!("no .def found in {spec}"))?;
    Ok(Arc::new(load_mugen_char(&vfs, chosen, 1, false)?))
}

pub fn inspect_moves(def: &CharDef) -> Vec<Value> {
    let mut out = Vec::new();
    for mv in &def.movelist {
        let anim_no = def
            .states
            .get(&mv.state)
            .and_then(|s| s.param("anim"))
            .and_then(|p| p.raw.trim().parse::<i32>().ok())
            .unwrap_or(mv.state);
        let (startup, active, recovery, total, max_reach) = match def.anims.get(&anim_no) {
            Some(a) => {
                let mut elapsed = 0i32;
                let mut first_hit = None;
                let mut act = 0i32;
                let mut reach = 0.0f32;
                for fr in &a.frames {
                    let t = fr.time.max(1);
                    if !fr.clsn1.is_empty() {
                        first_hit.get_or_insert(elapsed + 1);
                        act += t;
                        for b in &fr.clsn1 {
                            reach = reach.max(b[2]);
                        }
                    }
                    elapsed += t;
                }
                let su = first_hit.unwrap_or(elapsed);
                let rec = first_hit.map_or(0, |s| (elapsed - (s - 1 + act)).max(0));
                (su, act, rec, elapsed, reach)
            }
            None => (0, 0, 0, 0, 0.0),
        };
        out.push(json!({
            "name": mv.name,
            "state": mv.state,
            "command": mv.command,
            "cost": mv.cost,
            "startup": startup,
            "active": active,
            "recovery": recovery,
            "total": total,
            "reach": max_reach,
        }));
    }
    out
}

pub fn run_debug(opt: &DebugOptions) -> Result<Value, String> {
    let p1 = load_fighter(&opt.p1)?;
    let p2 = match &opt.p2 {
        Some(s) => load_fighter(s)?,
        None => p1.clone(),
    };
    let moves = inspect_moves(&p1);
    if opt.only_moves {
        return Ok(json!({ "fighter": p1.display_name, "moves": moves }));
    }
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(p1.clone(), 0, 8, 1);
    m.add_player(p2, 1, 8, 2);
    m.start_round();
    m.round_state = 2;
    m.fighters[0].ctrl = true;
    m.fighters[1].ctrl = true;
    let mut ai0 = Ai::new(8, 1);
    let mut ai1 = Ai::new(4, 2);
    let mut p1_states = BTreeSet::new();
    let mut helpers = BTreeSet::new();
    let mut cinema_events = 0usize;
    let mut super_events = 0usize;
    let mut peak_zoom = m.zoom;
    let mut hit_count = 0usize;
    for _ in 0..opt.frames.max(1) {
        let i0 = ai0.poll(&m, 0);
        let i1 = ai1.poll(&m, 1);
        m.tick(&[i0, i1]);
        peak_zoom = peak_zoom.max(m.zoom);
        if let Some(f) = m.fighters.first() {
            p1_states.insert(f.state_no);
        }
        for f in m.fighters.iter().filter(|f| !f.destroyed && f.is_helper) {
            helpers.insert((f.helper_id, f.state_no));
        }
        for ev in m.events.drain(..) {
            match ev {
                Event::Cinema { .. } => cinema_events += 1,
                Event::Super { .. } => super_events += 1,
                Event::Hit { .. } => hit_count += 1,
                _ => {}
            }
        }
    }
    let maps: Vec<Value> = m.fighters[0]
        .ikemen
        .map
        .iter()
        .map(|(k, v)| json!([k, v]))
        .collect();
    let console: Vec<String> = m.console.lines().cloned().collect();
    Ok(json!({
        "fighter": p1.display_name,
        "frames": opt.frames.max(1),
        "moves": moves,
        "trace": {
            "states": p1_states.into_iter().collect::<Vec<_>>(),
            "helpers": helpers.into_iter().map(|(id, st)| json!({"id": id, "state": st})).collect::<Vec<_>>(),
            "hits": hit_count,
            "supers": super_events,
            "cinemaEvents": cinema_events,
            "peakZoom": peak_zoom,
            "p1Hp": m.fighters[0].life,
            "p2Hp": m.fighters[1].life,
            "clipboard": m.fighters[0].clipboard,
            "maps": maps,
            "console": console,
            "warnings": m.warnings,
        }
    }))
}

pub fn format_text(v: &Value) -> String {
    let mut out = format!(
        "=== DEBUG-FIGHT: {} ===\n",
        v["fighter"].as_str().unwrap_or("?")
    );
    if let Some(arr) = v["moves"].as_array() {
        out.push_str(&format!("Moves ({}):\n", arr.len()));
        for mv in arr {
            out.push_str(&format!(
                "  S{:<4} {:<20} cost {:<4} su {:>2} act {:>2} rec {:>2} tot {:>3} reach {:.1}\n",
                mv["state"].as_i64().unwrap_or(0),
                mv["name"].as_str().unwrap_or(""),
                mv["cost"].as_f64().unwrap_or(0.0) as i32,
                mv["startup"].as_i64().unwrap_or(0),
                mv["active"].as_i64().unwrap_or(0),
                mv["recovery"].as_i64().unwrap_or(0),
                mv["total"].as_i64().unwrap_or(0),
                mv["reach"].as_f64().unwrap_or(0.0),
            ));
        }
    }
    if let Some(tr) = v.get("trace").filter(|t| !t.is_null()) {
        out.push_str(&format!(
            "Trace {}f: P1 HP {:.0} | P2 HP {:.0} | hits {} | supers {} | cinema {} | peakZoom {:.2}\n",
            v["frames"].as_u64().unwrap_or(0),
            tr["p1Hp"].as_f64().unwrap_or(0.0),
            tr["p2Hp"].as_f64().unwrap_or(0.0),
            tr["hits"].as_u64().unwrap_or(0),
            tr["supers"].as_u64().unwrap_or(0),
            tr["cinemaEvents"].as_u64().unwrap_or(0),
            tr["peakZoom"].as_f64().unwrap_or(1.0),
        ));
        out.push_str(&format!("States: {:?}\n", tr["states"]));
        if !tr["clipboard"].as_array().is_some_and(Vec::is_empty) {
            out.push_str(&format!("Clipboard: {}\n", tr["clipboard"]));
        }
        if !tr["console"].as_array().is_some_and(Vec::is_empty) {
            out.push_str(&format!("Console: {}\n", tr["console"]));
        }
        if !tr["warnings"].as_array().is_some_and(Vec::is_empty) {
            out.push_str(&format!("Warnings: {}\n", tr["warnings"]));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_fight_inspects_moves_and_traces_kaito() {
        let opt = DebugOptions {
            p1: "kaito".into(),
            p2: Some("akio".into()),
            frames: 90,
            only_moves: false,
        };
        let res = run_debug(&opt).expect("kaito vs akio trace ok");
        assert!(!res["moves"].as_array().unwrap().is_empty());
        assert!(res["trace"]["states"].as_array().unwrap().len() >= 1);
    }
}
