//! Load MUGEN characters from folders and run headless CPU-vs-CPU matches to validate compatibility.
//! Usage: cargo run --release -p ume-engine --example mugen_check -- [--strict] [--lint] <char_dir>...
use std::sync::Arc;
use ume_engine::ai::Ai;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::input::InputSource;
use ume_engine::loader::load_mugen_char;
use ume_engine::matchs::{Event, Match};
use ume_formats::vfs::{add_dir, find_char_defs, Vfs};

fn main() {
    let strict = std::env::args().any(|a| a == "--strict");
    let lint = std::env::args().any(|a| a == "--lint");
    let mut unsupported_chars = 0;
    for dir in std::env::args().skip(1).filter(|a| a != "--strict" && a != "--lint") {
        let mut vfs = Vfs::new();
        if let Err(e) = add_dir(&mut vfs, std::path::Path::new(&dir), "c") { println!("{dir}: io error {e}"); continue; }
        if lint { lint_report(&vfs, &dir); }
        for def in find_char_defs(&vfs) {
            let t = std::time::Instant::now();
            let d = match load_mugen_char(&vfs, &def, 1, true) { Ok(d) => Arc::new(d), Err(e) => { println!("FAIL load {def}: {e}"); continue; } };
            let load_ms = t.elapsed().as_millis();
            let diagnostics = d.diagnostics();
            if diagnostics.is_clean() { println!("DIAG {def}: clean"); } else {
                unsupported_chars += 1;
                println!("DIAG {def}: {}", diagnostics.summary());
                if std::env::var_os("TRACE").is_some() || strict { print!("{}", diagnostics.detail()); }
            }
            let res = std::panic::catch_unwind(|| {
                let mut m = Match::new(common_def(), None, flat_stage());
                m.add_player(d.clone(), 0, 6, 1); m.add_player(d.clone(), 1, 6, 2);
                m.start_round();
                let (mut a, mut b) = (Ai::new(6, 7), Ai::new(6, 13));
                let (mut hits, mut states) = (0, std::collections::BTreeSet::new());
                for _ in 0..60 * 60 {
                    let roots = m.root_indices();
                    let ia = a.poll(&m, roots[0]); let ib = b.poll(&m, roots[1]);
                    m.tick(&[ia, ib]);
                    for e in m.events.drain(..) { if matches!(e, Event::Hit { .. }) { hits += 1; } }
                    states.insert(m.fighters[roots[0]].state_no);
                    if std::env::var("TRACE").is_ok() && m.game_time % 20 == 0 && m.game_time < 600 { let f = &m.fighters[roots[0]]; println!("t={} rs={} st={} anim={} ctrl={} time={} type={}{} pos={:.0},{:.0}", m.game_time, m.round_state, f.state_no, f.anim_no, f.ctrl, f.state_time, f.state_type, f.move_type, f.pos.x, f.pos.y); }
                    if m.match_over { break; }
                }
                (hits, states.len(), m.round_no, m.wins)
            });
            match res {
                Ok((hits, ns, rn, w)) => println!("OK {} | name={:?} author={:?} anims={} states={} sprites={} load={}ms | 60s CPU fight: hits={hits} distinct_states={ns} round={rn} wins={w:?}", def, d.name, d.author, d.anims.len(), d.states.len(), d.sprites.len(), load_ms),
                Err(_) => println!("PANIC {def}"),
            }
        }
    }
    if strict && unsupported_chars > 0 {
        eprintln!("{unsupported_chars} character(s) use unsupported controllers, triggers or parameters");
        std::process::exit(1);
    }
}

fn lint_report(vfs: &Vfs, dir: &str) {
    let mut keys: Vec<&String> = vfs.keys().collect();
    keys.sort();
    let mut counts: std::collections::BTreeMap<ume_engine::lint::Rule, usize> = Default::default();
    let mut detail = String::new();
    for key in keys {
        let Some(file) = vfs.get(key) else { continue };
        for issue in ume_engine::lint::lint_file(key, &ume_formats::vfs::text(file.as_slice())) {
            *counts.entry(issue.rule).or_default() += 1;
            detail.push_str(&format!("  {}:{} {}: {}\n", issue.file, issue.line, issue.rule.label(), issue.message));
        }
    }
    if counts.is_empty() { println!("LINT {dir}: clean"); return; }
    let summary: Vec<String> = counts.iter().map(|(r, n)| format!("{n} {}", r.label())).collect();
    println!("LINT {dir}: {}", summary.join(", "));
    if std::env::var_os("TRACE").is_some() { print!("{detail}"); }
}
