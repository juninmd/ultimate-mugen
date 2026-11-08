mod support;
use std::collections::BTreeSet;
use std::io::Write;
use support::gogeta::*;
use support::render;

fn out_dir() -> Option<std::path::PathBuf> {
    let d = std::path::PathBuf::from(std::env::var("UME_EVIDENCE_DIR").ok()?);
    std::fs::create_dir_all(&d).ok()?;
    Some(d)
}

fn safe(n: &str) -> String {
    n.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

#[test]
fn contact_sheets() {
    let (Some(d), Some(out)) = (load(1), out_dir()) else {
        return;
    };
    let mut report = String::from("| command | input | entered states | fired | damage | recovered | sheet |\n|---|---|---|---|---|---|---|\n");
    for c in d.commands.iter() {
        if c.name.starts_with("ai") || c.name.starts_with("hold") {
            continue;
        }
        let mut best: Option<(i32, Vec<tiny_skia::Pixmap>, Outcome)> = None;
        for (air, gap, v10, v11) in [
            (false, 60.0f32, 0, 0),
            (false, 200.0, 0, 0),
            (true, 80.0, 0, 0),
            (false, 60.0, 5000, 5000),
        ] {
            let mut r = Duel::new(d.clone(), gap);
            let pm = r.m.fighters[0].power_max;
            r.m.fighters[0].power = if c.name == "charge" || c.name == "charge 1" {
                1000.0
            } else {
                pm
            };
            if air {
                r.jump();
            }
            r.m.fighters[0].vars[10] = v10;
            r.m.fighters[0].vars[11] = v11;
            let seq = synth(c);
            r.push(0, &seq);
            let life0 = r.m.fighters[1].life;
            let mut o = Outcome::default();
            let mut shots = vec![];
            let mut active_ticks = 0;
            for t in 0..260 {
                r.tick(1);
                if r.m.fighters[0].cmd.active.contains(c.name.as_str()) {
                    o.command_fired = true;
                }
                o.p1_states.insert(r.m.fighters[0].state_no);
                o.p2_states.insert(r.m.fighters[1].state_no);
                let busy = r.m.fighters[0].state_no != 0 && r.m.fighters[0].state_no != 11
                    || r.m.fighters[1].move_type == 'H';
                if busy {
                    active_ticks += 1;
                }
                if busy && t % 6 == 0 {
                    shots.push(render::frame(&r.m, true));
                }
            }
            o.dmg = life0 - r.m.fighters[1].life;
            r.m.fighters[1].pos.x = 300.0;
            r.tick(900);
            o.p1_recovered = r.m.fighters[0].ctrl && r.m.fighters[0].alive;
            o.p2_recovered = r.m.fighters[1].ctrl && r.m.fighters[1].alive;
            let score = o.p1_states.len() as i32
                + (o.dmg > 0.0) as i32 * 20
                + active_ticks / 10
                + o.command_fired as i32 * 5;
            if best.as_ref().is_none_or(|b| score > b.0) {
                best = Some((score, shots, o));
            }
        }
        let (_, mut shots, o) = best.unwrap();
        let step = (shots.len() as f32 / 8.0).max(1.0);
        let picked: Vec<_> = (0..8)
            .filter_map(|i| shots.get((i as f32 * step) as usize).cloned())
            .collect();
        shots.clear();
        let file = format!("{}.png", safe(&c.name));
        if !picked.is_empty() {
            render::sheet(&picked, 2).save_png(out.join(&file)).unwrap();
        }
        let states: Vec<_> = o
            .p1_states
            .iter()
            .filter(|s| **s != 0)
            .map(|s| s.to_string())
            .collect();
        report.push_str(&format!(
            "| {} | `{}` | {} | {} | {:.0} | {} | {} |\n",
            c.name,
            c.raw,
            states.join(" "),
            o.command_fired,
            o.dmg,
            o.p1_recovered && o.p2_recovered,
            if picked.is_empty() {
                "-".to_string()
            } else {
                file
            }
        ));
    }
    std::fs::File::create(out.join("commands.md"))
        .unwrap()
        .write_all(report.as_bytes())
        .unwrap();
    let _: BTreeSet<i32> = BTreeSet::new();
}
