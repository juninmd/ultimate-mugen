use super::{CharResult, Issue, Report, Severity, StageResult};
use serde_json::{json, Value};
use std::fmt::Write as _;
use std::path::Path;

const WASM_ATLAS_PIXELS: u64 = 2048 * 2048;
const LOAD_SLOW_MS: u128 = 2500;
const TICK_SLOW_US: f64 = 900.0;

pub fn perf_note(char: &CharResult) -> Option<Issue> {
    if char.sprite_pixels > WASM_ATLAS_PIXELS {
        return Some(Issue::new(
            Severity::Major,
            "perf",
            "char-wasm-atlas-overflow",
            &char.name,
            format!("{:.2} M sprite pixels ({} sprites) exceed the whole 2048 atlas ({:.1} M): the browser must evict and re-upload every frame", char.sprite_pixels as f64 / 1e6, char.sprites, WASM_ATLAS_PIXELS as f64 / 1e6),
        ));
    }
    if char.sprite_pixels > WASM_ATLAS_PIXELS / 2 {
        return Some(Issue::new(Severity::Minor, "perf", "char-wasm-atlas-pressure", &char.name, format!("{:.2} M sprite pixels is more than half of the 2048 web atlas", char.sprite_pixels as f64 / 1e6)));
    }
    if char.load_ms > LOAD_SLOW_MS {
        return Some(Issue::new(Severity::Minor, "perf", "char-load-slow", &char.name, format!("loading took {} ms", char.load_ms)));
    }
    None
}

fn mib(bytes: u64) -> f64 {
    bytes as f64 / 1048576.0
}

fn issue_json(issue: &Issue) -> Value {
    json!({
        "severity": issue.severity.label(),
        "category": issue.category,
        "code": issue.code,
        "subject": issue.subject,
        "detail": issue.detail,
    })
}

fn fight_json(fight: &super::FightResult) -> Value {
    json!({
        "stage": fight.stage,
        "opponent": fight.opponent,
        "ticks": fight.ticks,
        "hits": fight.hits,
        "damage": (fight.damage * 10.0).round() / 10.0,
        "distinct_states": fight.distinct_states,
        "stuck_ticks": fight.stuck_ticks,
        "tick_us": (fight.tick_us * 100.0).round() / 100.0,
        "verdict": fight.verdict,
    })
}

fn char_json(char: &CharResult) -> Value {
    json!({
        "def": char.def,
        "name": char.name,
        "author": char.author,
        "load_ms": char.load_ms,
        "anims": char.anims,
        "states": char.states,
        "sprites": char.sprites,
        "sounds": char.sounds,
        "sprite_mib": (mib(char.sprite_bytes) * 100.0).round() / 100.0,
        "issues": char.issues.iter().map(issue_json).collect::<Vec<_>>(),
        "fights": char.fights.iter().map(fight_json).collect::<Vec<_>>(),
    })
}

fn stage_json(stage: &StageResult) -> Value {
    json!({
        "def": stage.def,
        "name": stage.name,
        "load_ms": stage.load_ms,
        "layers": stage.layers,
        "sprites": stage.sprites,
        "sprite_mib": (mib(stage.sprite_bytes) * 100.0).round() / 100.0,
        "issues": stage.issues.iter().map(issue_json).collect::<Vec<_>>(),
        "fights": stage.fights.iter().map(fight_json).collect::<Vec<_>>(),
    })
}

fn tally(report: &Report) -> Value {
    let mut by_severity = [0usize; 4];
    let mut by_code: std::collections::BTreeMap<&str, usize> = Default::default();
    for issue in report.issues() {
        by_severity[match issue.severity {
            Severity::Critical => 0,
            Severity::Major => 1,
            Severity::Minor => 2,
            Severity::Info => 3,
        }] += 1;
        *by_code.entry(issue.code).or_default() += 1;
    }
    json!({
        "characters": report.chars.len(),
        "stages": report.stages.len(),
        "critical": by_severity[0],
        "major": by_severity[1],
        "minor": by_severity[2],
        "info": by_severity[3],
        "fights": report.chars.iter().map(|c| c.fights.len()).sum::<usize>() + report.stages.iter().map(|s| s.fights.len()).sum::<usize>(),
        "by_code": by_code,
    })
}

pub fn write_reports(report: &Report, out: &Path) -> Result<(), String> {
    std::fs::create_dir_all(out).map_err(|e| format!("{}: {e}", out.display()))?;
    let document = json!({
        "summary": tally(report),
        "characters": report.chars.iter().map(char_json).collect::<Vec<_>>(),
        "stages": report.stages.iter().map(stage_json).collect::<Vec<_>>(),
    });
    let json_text = serde_json::to_string_pretty(&document).map_err(|e| e.to_string())?;
    std::fs::write(out.join("report.json"), json_text + "\n").map_err(|e| e.to_string())?;

    let mut top: std::collections::BTreeMap<&str, Vec<Value>> = Default::default();
    for issue in report.issues() {
        let entry = top.entry(issue.code).or_default();
        if entry.len() < 5 {
            entry.push(json!({ "subject": issue.subject, "detail": issue.detail }));
        }
    }
    let summary = json!({ "summary": tally(report), "worst_by_code": top });
    std::fs::write(out.join("summary.json"), serde_json::to_string_pretty(&summary).map_err(|e| e.to_string())? + "\n").map_err(|e| e.to_string())?;

    let mut csv = String::from("severity,category,code,subject,detail\n");
    for issue in report.issues() {
        let _ = writeln!(csv, "{},{},{},\"{}\",\"{}\"", issue.severity.label(), issue.category, issue.code, issue.subject.replace('"', "'"), issue.detail.replace('"', "'"));
    }
    std::fs::write(out.join("issues.csv"), csv).map_err(|e| e.to_string())?;

    std::fs::write(out.join("report.md"), markdown(report)).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn console_lines(report: &Report) -> Vec<String> {
    let summary = tally(report);
    let mut lines = vec![
        format!(
            "roster audit: {} character(s), {} stage(s), {} fight(s)",
            summary["characters"], summary["stages"], summary["fights"]
        ),
        format!(
            "issues: {} critical, {} major, {} minor, {} info",
            summary["critical"], summary["major"], summary["minor"], summary["info"]
        ),
    ];
    if let Some(codes) = summary["by_code"].as_object() {
        for (code, count) in codes {
            lines.push(format!("  {count:>5}  {code}"));
        }
    }
    lines
}

fn markdown(report: &Report) -> String {
    let summary = tally(report);
    let mut md = String::new();
    let _ = writeln!(md, "# Homologation report\n");
    let _ = writeln!(md, "| metric | value |\n|---|---|");
    let _ = writeln!(md, "| characters | {} |", summary["characters"]);
    let _ = writeln!(md, "| stages | {} |", summary["stages"]);
    let _ = writeln!(md, "| fights | {} |", summary["fights"]);
    let _ = writeln!(md, "| critical | {} |", summary["critical"]);
    let _ = writeln!(md, "| major | {} |", summary["major"]);
    let _ = writeln!(md, "| minor | {} |", summary["minor"]);
    let _ = writeln!(md, "| info | {} |", summary["info"]);
    let _ = writeln!(md, "\n## Characters with the most problems\n");
    let _ = writeln!(md, "| character | severity | issues | fights | hits |");
    let _ = writeln!(md, "|---|---|---|---|---|");
    let mut ranked: Vec<&CharResult> = report.chars.iter().filter(|c| !c.issues.is_empty()).collect();
    ranked.sort_by_key(|c| (c.issues.iter().map(|i| i.severity as u8).min().unwrap_or(9), usize::MAX - c.issues.len()));
    for char in ranked.iter().take(40) {
        let hits: u32 = char.fights.iter().map(|f| f.hits).sum();
        let worst = char.issues.iter().map(|i| i.severity).min().unwrap_or(Severity::Info);
        let _ = writeln!(md, "| {} | {} | {} | {} | {} |", char.name, worst.label(), char.issues.len(), char.fights.len(), hits);
    }
    let _ = writeln!(md, "\n## Stage problems\n");
    let _ = writeln!(md, "| stage | severity | issues | layers |");
    let _ = writeln!(md, "|---|---|---|---|");
    let mut stages: Vec<&StageResult> = report.stages.iter().filter(|s| !s.issues.is_empty()).collect();
    stages.sort_by_key(|s| (s.issues.iter().map(|i| i.severity as u8).min().unwrap_or(9), usize::MAX - s.issues.len()));
    for stage in stages {
        let worst = stage.issues.iter().map(|i| i.severity).min().unwrap_or(Severity::Info);
        let _ = writeln!(md, "| {} | {} | {} | {} |", stage.name, worst.label(), stage.issues.len(), stage.layers);
    }
    let _ = writeln!(md, "\n## Slowest loading packs\n");
    let _ = writeln!(md, "| character | load ms | sprite MiB | states |");
    let _ = writeln!(md, "|---|---|---|---|");
    let mut slow: Vec<&CharResult> = report.chars.iter().collect();
    slow.sort_by_key(|c| u128::MAX - c.load_ms);
    for char in slow.iter().take(12) {
        let _ = writeln!(md, "| {} | {} | {:.1} | {} |", char.name, char.load_ms, mib(char.sprite_bytes), char.states);
    }
    let _ = writeln!(md, "\n## Slowest simulation ticks\n");
    let _ = writeln!(md, "| character | opponent | stage | us/tick | hits |");
    let _ = writeln!(md, "|---|---|---|---|---|");
    let mut ticks: Vec<(&CharResult, &super::FightResult)> = report.chars.iter().flat_map(|c| c.fights.iter().map(move |f| (c, f))).collect();
    ticks.sort_by(|a, b| b.1.tick_us.partial_cmp(&a.1.tick_us).unwrap_or(std::cmp::Ordering::Equal));
    for (char, fight) in ticks.iter().take(12) {
        let _ = writeln!(md, "| {} | {} | {} | {:.0} | {} |", char.name, fight.opponent, fight.stage, fight.tick_us, fight.hits);
    }
    md
}
