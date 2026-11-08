mod support;
use std::collections::{BTreeMap, BTreeSet};
use support::gogeta::*;

fn entries() -> Vec<(i32, Vec<String>)> {
    let text = std::fs::read(dir().unwrap().join("gogeta-ssj4.cmd")).unwrap();
    let text: String = text
        .iter()
        .map(|&b| b as char)
        .collect::<String>()
        .replace('\r', "");
    let from = text.find("[Statedef -1]").unwrap();
    let mut out = vec![];
    for block in text[from..].split("\n[State ").skip(1) {
        let mut value = None;
        let mut cmds = BTreeSet::new();
        for line in block.lines() {
            let l = line.split(';').next().unwrap().trim();
            let low = l.to_lowercase();
            if let Some(v) = low.strip_prefix("value") {
                if let Some(v) = v.trim().strip_prefix('=') {
                    if let Ok(n) = v.trim().parse::<i32>() {
                        value = Some(n);
                    }
                }
            }
            let mut rest = l;
            if low.contains("!=") { continue; }
            while let Some(p) = rest.to_lowercase().find("command") {
                rest = &rest[p + 7..];
                if let Some(q) = rest.find('"') {
                    let r2 = &rest[q + 1..];
                    if let Some(e) = r2.find('"') {
                        cmds.insert(r2[..e].to_string());
                        rest = &r2[e + 1..];
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
        if let Some(v) = value {
            out.push((v, cmds.into_iter().collect()));
        }
    }
    out
}

fn overlay(d: &std::sync::Arc<ume_engine::chardef::CharDef>, names: &[String]) -> Vec<u32> {
    let seqs: Vec<Vec<u32>> = names
        .iter()
        .filter_map(|n| d.commands.iter().find(|c| &c.name == n))
        .map(synth)
        .collect();
    let len = seqs.iter().map(|s| s.len()).max().unwrap_or(0);
    let mut out = vec![0u32; len];
    for s in &seqs {
        let off = len - s.len();
        for (i, b) in s.iter().enumerate() {
            out[off + i] |= *b;
        }
    }
    out
}

#[test]
fn every_special_entry_is_reachable() {
    let Some(d) = load(1) else { return };
    let mut report: BTreeMap<i32, (Vec<String>, bool, String)> = BTreeMap::new();
    for (target, names) in entries() {
        if names.is_empty()
            || names
                .iter()
                .all(|n| n.starts_with("hold") && names.len() == 1)
        {
            continue;
        }
        let seq = overlay(&d, &names);
        let mut hit = false;
        let mut how = String::new();
        'outer: for (air, v10, v11) in [
            (false, 0, 0),
            (false, 0, 5000),
            (false, 5000, 5000),
            (true, 0, 0),
            (true, 0, 5000),
        ] {
            for gap in [40.0f32, 200.0] {
                let mut r = Duel::new(d.clone(), gap);
                let pm = r.m.fighters[0].power_max;
                r.m.fighters[0].power = if target == 730 { 5000.0 } else { pm };
                r.m.fighters[0].vars[10] = v10;
                r.m.fighters[0].vars[11] = v11;
                if air {
                    r.jump();
                    r.m.fighters[0].vars[10] = v10;
                    r.m.fighters[0].vars[11] = v11;
                    r.m.fighters[0].power = if target == 730 { 5000.0 } else { pm };
                }
                r.push(0, &seq);
                for _ in 0..(seq.len() + 40) {
                    r.tick(1);
                    if r.m.fighters[0].state_no == target {
                        hit = true;
                        how = format!("air={air} var10={v10} var11={v11} gap={gap}");
                        break 'outer;
                    }
                }
            }
        }
        report.insert(target, (names, hit, how));
    }
    let mut bad = 0;
    for (t, (n, ok, how)) in &report {
        println!("ENTRY {t:>6} {n:?} reached={ok} {how}");
        if !ok {
            bad += 1;
        }
    }
    println!("ENTRY unreachable: {bad} of {}", report.len());
    assert_eq!(bad, 0);
}
