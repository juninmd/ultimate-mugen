mod run;

use run::{run_pair, Verdict};
use std::path::Path;
use std::sync::Arc;
use ume_engine::chardef::CharDef;
use ume_engine::loader::load_mugen_char;
use ume_formats::vfs::{add_dir, find_char_defs, Vfs};

struct Loaded {
    def: String,
    data: Arc<CharDef>,
}

fn load_all(dirs: &[String], failures: &mut Vec<String>) -> Vec<Loaded> {
    let mut loaded = Vec::new();
    for dir in dirs {
        let mut vfs = Vfs::new();
        if let Err(e) = add_dir(&mut vfs, Path::new(dir), "c") {
            failures.push(format!("{dir}: io error {e}"));
            continue;
        }
        for def in find_char_defs(&vfs) {
            match load_mugen_char(&vfs, &def, 1, true) {
                Ok(data) => loaded.push(Loaded {
                    def,
                    data: Arc::new(data),
                }),
                Err(e) => failures.push(format!("{def}: {e}")),
            }
        }
    }
    loaded
}

fn pairs(count: usize, matrix: bool) -> Vec<(usize, usize)> {
    if matrix {
        (0..count)
            .flat_map(|a| (0..count).map(move |b| (a, b)))
            .collect()
    } else {
        (0..count).map(|a| (a, 0)).collect()
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let strict = args.iter().any(|a| a == "--strict");
    let matrix = args.iter().any(|a| a == "--matrix");
    let dirs: Vec<String> = args
        .iter()
        .filter(|a| !a.starts_with("--"))
        .cloned()
        .collect();

    let mut failures = Vec::new();
    let loaded = load_all(&dirs, &mut failures);
    for f in &failures {
        println!("LOAD   {f}");
    }
    if loaded.is_empty() {
        println!("no characters loaded");
        std::process::exit(if strict || !failures.is_empty() { 1 } else { 0 });
    }

    let (mut bad, mut total) = (failures.len(), 0usize);
    for (a, b) in pairs(loaded.len(), matrix) {
        let report = run_pair(loaded[a].data.clone(), loaded[b].data.clone());
        total += 1;
        if report.verdict != Verdict::Ok {
            bad += 1;
        }
        println!(
            "{:<6} {} vs {}: ticks={} hits={} states={} worst_stuck_ticks={}",
            report.verdict.label(),
            loaded[a].def,
            loaded[b].def,
            report.ticks,
            report.hits,
            report.distinct_states,
            report.stuck_ticks
        );
    }
    println!("{total} fight(s), {bad} problem(s)");
    if strict && bad > 0 {
        std::process::exit(1);
    }
}
