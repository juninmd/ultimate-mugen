mod char_audit;
mod fight;
mod model;
mod panic_capture;
mod report;
mod schedule;
mod stage_audit;

pub use model::{CharResult, FightResult, Issue, Job, Options, Report, Severity, StageResult};
pub use report::{console_lines, write_reports};

use crate::roster_audit::schedule::Plan;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use ume_engine::chardef::CharDef;
use ume_engine::loader::load_mugen_char;
use ume_formats::stage::MugenStage;
use ume_formats::vfs::{add_dir, find_char_defs, find_stage_defs, Vfs};

fn def_dirs(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        let mut has_def = false;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("def")) {
                has_def = true;
            }
        }
        if has_def {
            out.push(dir);
        }
    }
    out.sort();
    out
}

fn scan(root: &Path, stage: bool) -> Result<Vec<Job>, String> {
    let mut jobs = Vec::new();
    let mut seen: std::collections::HashSet<String> = Default::default();
    for dir in def_dirs(root) {
        let mut vfs = Vfs::new();
        if add_dir(&mut vfs, &dir, "c").is_err() {
            continue;
        }
        let defs = if stage { find_stage_defs(&vfs) } else { find_char_defs(&vfs) };
        for def in defs {
            if !seen.insert(def.clone()) {
                continue;
            }
            jobs.push(Job { def, dir: dir.display().to_string() });
        }
    }
    Ok(jobs)
}

fn load_char(dir: &str, def: &str) -> Result<CharDef, String> {
    let mut vfs = Vfs::new();
    add_dir(&mut vfs, Path::new(dir), "c").map_err(|e| format!("{dir}: {e}"))?;
    load_mugen_char(&vfs, def, 1, true)
}

fn health_issues(job: &Job) -> Vec<Issue> {
    let mut vfs = Vfs::new();
    if add_dir(&mut vfs, Path::new(&job.dir), "c").is_err() {
        return Vec::new();
    }
    ume_formats::health::problems(&vfs, &job.def)
        .into_iter()
        .map(|problem| Issue::new(Severity::Critical, "web", "char-web-quarantine", &job.def, problem))
        .collect()
}

pub fn run(opts: &Options) -> Result<Report, String> {
    panic_capture::install();
    let all_chars = scan(&opts.chars_root, false)?;
    let all_stages = scan(&opts.stages_root, true)?;
    let mut char_jobs = all_chars.clone();
    let mut stage_jobs = all_stages.clone();
    if let Some(needle) = &opts.filter {
        let needle = needle.to_lowercase();
        char_jobs.retain(|j| j.def.to_lowercase().contains(&needle) || j.dir.to_lowercase().contains(&needle));
    }
    if let Some(needle) = &opts.stage_filter {
        let needle = needle.to_lowercase();
        stage_jobs.retain(|j| j.def.to_lowercase().contains(&needle));
    }
    if let Some(limit) = opts.chars {
        char_jobs.truncate(limit);
    }
    if let Some(limit) = opts.stages {
        stage_jobs.truncate(limit);
    }
    let plan = Arc::new(Plan::new(&all_chars, &all_stages, opts));
    let mut report = Report::default();
    report.chars = ume_formats::par::map(&char_jobs, |job| audit_char(job, &plan, opts));
    report.stages = ume_formats::par::map(&stage_jobs, |job| audit_stage(job, &plan, opts));
    Ok(report)
}

fn audit_char(job: &Job, plan: &Arc<Plan>, opts: &Options) -> CharResult {
    let mut out = CharResult { def: job.def.clone(), ..Default::default() };
    out.issues.extend(health_issues(job));
    let started = std::time::Instant::now();
    let loaded = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| load_char(&job.dir, &job.def))) {
        Ok(Ok(def)) => def,
        Ok(Err(error)) => {
            out.issues.push(Issue::new(Severity::Critical, "load", "char-load-fail", &job.def, error));
            return out;
        }
        Err(_) => {
            out.issues.push(Issue::new(Severity::Critical, "load", "char-load-panic", &job.def, panic_capture::take()));
            return out;
        }
    };
    out.load_ms = started.elapsed().as_millis();
    out.name = loaded.name.clone();
    out.author = loaded.author.clone();
    out.anims = loaded.anims.len();
    out.states = loaded.states.len();
    out.sprites = loaded.sprites.len();
    out.sounds = loaded.sounds.len();
    let def = Arc::new(loaded);
    out.sprite_pixels = def.sprites.values().map(|s| (s.width.max(0.0) as u64) * (s.height.max(0.0) as u64)).sum();
    out.sprite_bytes = out.sprite_pixels * 4;
    out.issues.extend(char_audit::static_issues(&def));
    if let Some(note) = report::perf_note(&out) {
        out.issues.push(note);
    }
    if !opts.skip_fights {
        for (index, (foe_def, foe_dir, stage_def, stage_dir)) in plan.foes(&job.def).iter().enumerate() {
            if index >= opts.fights {
                break;
            }
            let Some(foe) = plan_load(foe_def, foe_dir) else { continue };
            let stage = plan_load_stage(stage_def, stage_dir);
            let probe = fight::run(def.clone(), foe, stage, 6 + index as i32);
            out.fights.push(probe.result);
            out.issues.extend(probe.issues);
        }
    }
    if !opts.quiet {
        eprintln!("CHAR {} | {} issue(s)", job.def, out.issues.len());
    }
    out
}

fn audit_stage(job: &Job, plan: &Arc<Plan>, opts: &Options) -> StageResult {
    let mut out = StageResult { def: job.def.clone(), ..Default::default() };
    let started = std::time::Instant::now();
    let loaded = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| stage_audit::load(&job.dir, &job.def))) {
        Ok(Ok(stage)) => stage,
        Ok(Err(error)) => {
            out.issues.push(Issue::new(Severity::Critical, "stage", "stage-load-fail", &job.def, error));
            return out;
        }
        Err(_) => {
            out.issues.push(Issue::new(Severity::Critical, "stage", "stage-load-panic", &job.def, panic_capture::take()));
            return out;
        }
    };
    out.load_ms = started.elapsed().as_millis();
    out.name = loaded.stage.name.clone();
    out.layers = loaded.stage.layers.len();
    out.sprites = loaded.stage.sprites.len();
    out.sprite_bytes = loaded.stage.sprites.values().map(|s| s.rgba.len() as u64).sum();
    out.issues.extend(loaded.issues);
    if !opts.skip_fights {
        let (a_def, a_dir, b_def, b_dir) = plan.stage_pair(&job.def);
        if let (Some(a), Some(b)) = (plan_load(&a_def, &a_dir), plan_load(&b_def, &b_dir)) {
            let probe = fight::run_on_stage(a, b, Arc::new(loaded.stage), 11, &job.def);
            out.fights.push(probe.result);
            out.issues.extend(probe.issues);
        }
    }
    if !opts.quiet {
        eprintln!("STAGE {} | {} issue(s)", job.def, out.issues.len());
    }
    out
}

fn plan_load(def: &str, dir: &str) -> Option<Arc<CharDef>> {
    load_char(dir, def).ok().map(Arc::new)
}

fn plan_load_stage(def: &str, dir: &str) -> Option<Arc<MugenStage>> {
    stage_audit::load(dir, def).ok().map(|loaded| Arc::new(loaded.stage))
}
