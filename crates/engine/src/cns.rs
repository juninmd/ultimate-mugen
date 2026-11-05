//! CNS/CMD text → state definitions, constants and commands.
use crate::chardef::{next_uid, safe_expr, Controller, Param, StateDef};
use crate::commands::{parse_command, CommandDef};
use std::collections::{HashMap, HashSet};
use ume_formats::ini::{parse_ini_no_lines, unquote, Section};

pub fn build_controller(s: &Section) -> Option<Controller> {
    let kind = s.get("type")?.trim().to_ascii_lowercase();
    if kind.is_empty() { return None; }
    let mut c = Controller { kind, label: s.name.clone(), trigger_all: vec![], triggers: vec![], params: vec![], persistent: 1, ignore_hitpause: false, uid: next_uid() };
    for (k, v) in &s.entries {
        if k == "type" { continue; }
        if k == "triggerall" { c.trigger_all.push(safe_expr(v)); continue; }
        if let Some(n) = k.strip_prefix("trigger").and_then(|n| n.parse::<usize>().ok()) {
            if n == 0 || n > 64 { continue; }
            while c.triggers.len() < n { c.triggers.push(vec![]); }
            c.triggers[n - 1].push(safe_expr(v));
            continue;
        }
        if k == "persistent" { c.persistent = v.trim().parse().unwrap_or(1); continue; }
        if k == "ignorehitpause" { c.ignore_hitpause = v.trim().parse::<i32>().unwrap_or(0) != 0; }
        c.params.push((k.clone(), Param::new(v)));
    }
    if let Some(i) = c.triggers.iter().position(|t| t.is_empty()) { c.triggers.truncate(i); }
    Some(c)
}

pub fn parse_states(text: &str, into: &mut HashMap<i32, StateDef>, override_existing: bool) {
    let secs = parse_ini_no_lines(text);
    parse_states_from_sections(&secs, into, override_existing);
}

pub fn parse_states_from_sections(secs: &[Section], into: &mut HashMap<i32, StateDef>, override_existing: bool) {
    let mut cur: Option<i32> = None;
    let mut skip = false;
    for s in secs {
        let ln = s.lname.trim();
        if let Some(rest) = ln.strip_prefix("statedef") {
            let id: i32 = rest.trim().split(|c: char| !(c == '-' || c.is_ascii_digit())).next().and_then(|x| x.parse().ok()).unwrap_or(i32::MIN);
            if id == i32::MIN { cur = None; continue; }
            if into.contains_key(&id) && !override_existing && id >= 0 { skip = true; cur = None; continue; }
            skip = false;
            if !(id < 0 && into.contains_key(&id)) {
                into.insert(id, StateDef { id, params: vec![], controllers: vec![] });
            }
            let sd = into.get_mut(&id).unwrap();
            for (k, v) in &s.entries { sd.params.push((k.clone(), Param::new(v))); }
            cur = Some(id);
            continue;
        }
        if ln.starts_with("state ") || ln.starts_with("state\t") || (ln.starts_with("state") && ln[5..].trim_start().starts_with(|c: char| c == '-' || c.is_ascii_digit())) {
            if skip { continue; }
            if let (Some(id), Some(c)) = (cur, build_controller(s)) { into.get_mut(&id).unwrap().controllers.push(c); }
            continue;
        }
        cur = None; skip = false;
    }
}

pub fn parse_constants(text: &str, into: &mut HashMap<String, f64>, provided: &mut HashSet<String>) {
    let secs = parse_ini_no_lines(text);
    parse_constants_from_sections(&secs, into, provided);
}

pub fn parse_constants_from_sections(secs: &[Section], into: &mut HashMap<String, f64>, provided: &mut HashSet<String>) {
    for s in secs {
        let sec = s.lname.as_str();
        if !["data", "size", "velocity", "movement", "constants"].contains(&sec) { continue; }
        for (k, v) in &s.entries {
            let parts: Vec<f64> = v.split(',').map(|x| x.trim().parse::<f64>().unwrap_or(f64::NAN)).collect();
            let key = if sec == "constants" { k.to_string() } else { format!("{sec}.{k}") };
            provided.insert(key.clone());
            if parts.len() == 1 {
                let val = if parts[0].is_finite() { parts[0] } else { 0.0 };
                into.insert(key.clone(), val);
                if sec == "velocity" { into.insert(format!("{key}.x"), val); provided.insert(format!("{key}.x")); }
            } else {
                for (i, p) in parts.iter().enumerate() {
                    let c = ["x", "y", "z"].get(i).copied().unwrap_or("w");
                    let kk = format!("{key}.{c}");
                    into.insert(kk.clone(), if p.is_finite() { *p } else { 0.0 });
                    provided.insert(kk);
                }
            }
        }
    }
    for (to, from) in [("velocity.jump.y", "velocity.jump.neu.y"), ("velocity.runjump.y", "velocity.runjump.fwd.y"), ("velocity.airjump.y", "velocity.airjump.neu.y")] {
        if provided.contains(from) { if let Some(v) = into.get(from).copied() { into.insert(to.into(), v); provided.insert(to.into()); } }
    }
}

pub fn scale_default_constants(into: &mut HashMap<String, f64>, provided: &HashSet<String>, factor: f64) {
    if factor == 1.0 { return; }
    for (k, v) in into.iter_mut() {
        if provided.contains(k) || k.contains("xscale") || k.contains("yscale") || k.ends_with("friction") || k.contains("airjump.num") || k.starts_with("data.") { continue; }
        if k.starts_with("size.") || k.starts_with("velocity.") || k.starts_with("movement.") { *v *= factor; }
    }
}

fn pause_buffer_of(s: &Section) -> Option<i32> {
    ["pausebuffer", "command.pausebuffer", "buffer.pausetime", "command.buffer.pausetime"].iter().find_map(|k| s.get(k).and_then(|v| v.trim().parse().ok()))
}

pub fn parse_cmd(text: &str) -> (Vec<CommandDef>, HashMap<i32, StateDef>) {
    let secs = parse_ini_no_lines(text);
    let (mut dt, mut db, mut dp) = (15, 1, 0);
    if let Some(d) = secs.iter().find(|s| s.lname == "defaults") {
        dt = d.get("command.time").and_then(|v| v.trim().parse().ok()).unwrap_or(15);
        db = d.get("command.buffer.time").and_then(|v| v.trim().parse().ok()).unwrap_or(1);
        dp = pause_buffer_of(d).unwrap_or(0);
    }
    let mut cmds = vec![];
    for s in &secs {
        if s.lname != "command" { continue; }
        let (Some(n), Some(c)) = (s.get("name"), s.get("command")) else { continue };
        let name = unquote(n);
        if name.is_empty() { continue; }
        let t = s.get("time").and_then(|v| v.trim().parse().ok()).unwrap_or(dt);
        let b = s.get("buffer.time").and_then(|v| v.trim().parse().ok()).unwrap_or(db);
        let mut def = parse_command(&name, &unquote(c), t, b);
        def.pause_buffer = pause_buffer_of(s).unwrap_or(dp).max(0);
        cmds.push(def);
    }
    let mut states = HashMap::new();
    parse_states_from_sections(&secs, &mut states, true);
    (cmds, states)
}
