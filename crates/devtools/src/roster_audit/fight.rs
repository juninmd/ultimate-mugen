use super::{FightResult, Issue, Severity};
use crate::roster_audit::stage_audit;
use std::collections::BTreeSet;
use std::sync::Arc;
use ume_engine::ai::Ai;
use ume_engine::chardef::CharDef;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::input::InputSource;
use ume_engine::matchs::{Event, Match};
use ume_engine::stage_bg::BgState;
use ume_formats::stage::MugenStage;

const ROUND_TICKS: usize = 60 * 30;
const LONGEST_OFF_FLOOR: i32 = 20;
const FLOOR_DEPTH_MARGIN: f32 = 12.0;
const LONGEST_OFF_CAMERA: i32 = 3;
const LONGEST_WITHOUT_CONTROL: i32 = 420;
const LONGEST_ATTACK: i32 = 240;
const OFF_CAMERA_MARGIN: f32 = 10.0;
const LYING_STATES: [i32; 5] = [5110, 5120, 5150, 5170, 5900];

pub struct Probe {
    pub result: FightResult,
    pub issues: Vec<Issue>,
}

#[derive(Default, Clone, Copy)]
struct Streak {
    sunk: i32,
    sunk_depth: f32,
    hidden: i32,
    no_control: i32,
    attacking: i32,
    attack_state: i32,
}

pub fn run(a: Arc<CharDef>, b: Arc<CharDef>, stage: Option<Arc<MugenStage>>, seed: i32) -> Probe {
    let mut issues = Vec::new();
    let info = match &stage {
        Some(stage) => stage_audit::info(stage),
        None => flat_stage(),
    };
    let label = stage.as_ref().map(|s| s.name.clone()).unwrap_or_else(|| "flat".to_string());
    let pair = format!("{} vs {}", a.name, b.name);
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| fight(a, b, stage, info, seed, &label, &mut issues)));
    match outcome {
        Ok(result) => Probe { result, issues },
        Err(_) => {
            let message = crate::roster_audit::panic_capture::take();
            issues.push(Issue::new(Severity::Critical, "mechanics", "fight-panic", &label, format!("{pair} (seed {seed}): {message}")));
            Probe { result: FightResult { verdict: "panic", stage: label, opponent: pair, ..Default::default() }, issues }
        }
    }
}

pub fn run_on_stage(a: Arc<CharDef>, b: Arc<CharDef>, stage: Arc<MugenStage>, seed: i32, def: &str) -> Probe {
    let mut probe = run(a, b, Some(stage), seed);
    for issue in &mut probe.issues {
        if issue.subject.is_empty() || issue.subject == "flat" {
            issue.subject = def.to_string();
        }
    }
    probe.result.stage = def.to_string();
    probe
}

fn fight(
    a: Arc<CharDef>,
    b: Arc<CharDef>,
    stage: Option<Arc<MugenStage>>,
    info: ume_engine::matchs::StageInfo,
    seed: i32,
    label: &str,
    issues: &mut Vec<Issue>,
) -> FightResult {
    let opponent = b.name.clone();
    let mut m = Match::new(common_def(), None, info);
    m.add_player(a.clone(), 0, 6, 1);
    m.add_player(b.clone(), 1, 6, 2);
    if let Some(stage) = stage {
        m.bg = Some(BgState::new(stage));
    }
    m.start_round();
    let (mut ai_a, mut ai_b) = (Ai::new(6, seed as u32), Ai::new(6, (seed * 7 + 13) as u32));
    let roots = m.root_indices();
    let mut streaks = [Streak::default(); 2];
    let mut reported: BTreeSet<&'static str> = BTreeSet::new();
    let mut hits = 0u32;
    let mut damage = 0.0f32;
    let mut states: BTreeSet<i32> = BTreeSet::new();
    let mut tick_total = std::time::Duration::ZERO;
    let mut ticks = 0u32;

    for _ in 0..ROUND_TICKS {
        ticks += 1;
        let inputs = [ai_a.poll(&m, roots[0]), ai_b.poll(&m, roots[1])];
        let began = std::time::Instant::now();
        m.tick(&inputs);
        tick_total += began.elapsed();
        for event in m.events.drain(..) {
            if let Event::Hit { damage: d, .. } = event {
                hits += 1;
                damage += d;
            }
        }
        check_camera(&m, label, issues, &mut reported);
        let in_fight = m.round_state == 2;
        for side in 0..2 {
            let f = &m.fighters[roots[side]];
            states.insert(f.state_no);
            if in_fight {
                check_fighter(&m, f, side, &mut streaks[side], label, issues, &mut reported);
            }
        }
        if m.match_over {
            break;
        }
    }
    check_contact(hits, &a, &b, label, issues, &mut reported);
    check_rescues(&m, &roots, label, issues);
    check_console(&m, label, issues);
    let verdict = if reported.is_empty() { "ok" } else { "problem" };
    FightResult {
        stage: label.to_string(),
        opponent,
        ticks,
        hits,
        damage,
        distinct_states: states.len(),
        stuck_ticks: streaks.iter().map(|s| s.no_control).max().unwrap_or(0),
        tick_us: if ticks == 0 { 0.0 } else { tick_total.as_secs_f64() * 1e6 / ticks as f64 },
        verdict,
    }
}

fn note(issues: &mut Vec<Issue>, reported: &mut BTreeSet<&'static str>, code: &'static str, severity: Severity, subject: &str, detail: String) {
    if !reported.insert(code) {
        return;
    }
    issues.push(Issue::new(severity, "mechanics", code, subject, detail));
}

fn check_camera(m: &Match, label: &str, issues: &mut Vec<Issue>, reported: &mut BTreeSet<&'static str>) {
    if !(m.camera.x.is_finite() && m.camera.y.is_finite() && m.zoom.is_finite()) {
        note(issues, reported, "fight-camera-nan", Severity::Critical, label, format!("camera {:?} zoom {}", m.camera, m.zoom));
    }
}

fn check_fighter(
    m: &Match,
    f: &ume_engine::fighter::Fighter,
    side: usize,
    s: &mut Streak,
    label: &str,
    issues: &mut Vec<Issue>,
    reported: &mut BTreeSet<&'static str>,
) {
    let suffix = if side == 0 { "p1" } else { "p2" };
    let subject = format!("{label} {suffix}");
    let finite = f.pos.x.is_finite() && f.pos.y.is_finite() && f.vel.x.is_finite() && f.vel.y.is_finite() && f.life.is_finite() && f.power.is_finite();
    if !finite {
        note(issues, reported, "fight-non-finite", Severity::Critical, &subject, format!("pos {:?} vel {:?} life {} power {}", f.pos, f.vel, f.life, f.power));
        return;
    }
    if f.life < 0.0 || f.life > f.life_max + 0.5 {
        note(issues, reported, "fight-life-range", Severity::Major, &subject, format!("life {} of {}", f.life, f.life_max));
    }
    if f.power < 0.0 || f.power > f.power_max + 0.5 {
        note(issues, reported, "fight-power-range", Severity::Major, &subject, format!("power {} of {}", f.power, f.power_max));
    }
    let frozen = m.pause.is_some() || f.hit_pause > 0 || f.hit_shake > 0 || f.bind.is_some();
    s.sunk = if f.pos.y <= 1.5 { 0 } else if frozen { s.sunk } else { s.sunk + 1 };
    s.sunk_depth = if s.sunk == 0 { 0.0 } else { s.sunk_depth.max(f.pos.y) };
    if s.sunk > LONGEST_OFF_FLOOR && s.sunk_depth > FLOOR_DEPTH_MARGIN {
        note(issues, reported, if side == 0 { "fight-below-floor-p1" } else { "fight-below-floor-p2" }, Severity::Major, &subject, format!("y {:.1} for {} ticks in state {}", s.sunk_depth, s.sunk, f.state_no));
        s.sunk = 0;
        s.sunk_depth = 0.0;
    }
    let outside = (f.pos.x - m.camera.x).abs() > m.view_w / 2.0 + OFF_CAMERA_MARGIN;
    s.hidden = if outside { s.hidden + 1 } else { 0 };
    if s.hidden > LONGEST_OFF_CAMERA {
        note(issues, reported, if side == 0 { "fight-off-camera-p1" } else { "fight-off-camera-p2" }, Severity::Major, &subject, format!("x {:.0} while the camera is at {:.0} in state {}", f.pos.x, m.camera.x, f.state_no));
    }
    s.no_control = if f.alive && !f.ctrl && !frozen && !LYING_STATES.contains(&f.state_no) { s.no_control + 1 } else { 0 };
    s.attacking = if f.alive && f.move_type == 'A' && f.state_no == s.attack_state && !frozen { s.attacking + 1 } else { 0 };
    s.attack_state = f.state_no;
    if s.no_control == LONGEST_WITHOUT_CONTROL + 1 {
        note(issues, reported, if side == 0 { "fight-stuck-p1" } else { "fight-stuck-p2" }, Severity::Major, &subject, format!("{} ticks without control in state {} (anim {})", s.no_control, f.state_no, f.anim_no));
    }
    if s.attacking == LONGEST_ATTACK + 1 {
        note(issues, reported, if side == 0 { "fight-endless-attack-p1" } else { "fight-endless-attack-p2" }, Severity::Major, &subject, format!("{} ticks in attack state {}", s.attacking, f.state_no));
    }
}

fn check_contact(hits: u32, a: &CharDef, b: &CharDef, label: &str, issues: &mut Vec<Issue>, reported: &mut BTreeSet<&'static str>) {
    if hits > 0 {
        return;
    }
    note(issues, reported, "fight-no-contact", Severity::Major, label, format!("{} vs {} never landed a hit in 30s (anims {} / {})", a.name, b.name, a.anims.len(), b.anims.len()));
}

fn check_rescues(m: &Match, roots: &[usize], label: &str, issues: &mut Vec<Issue>) {
    for (side, root) in roots.iter().enumerate() {
        let f = &m.fighters[*root];
        if f.ikemen.rescued == 0 {
            continue;
        }
        let tag = if side == 0 { "p1" } else { "p2" };
        issues.push(Issue::new(
            Severity::Minor,
            "mechanics",
            "fight-watchdog-rescue",
            format!("{label} {tag}"),
            format!("{}: the engine's own fallback intervened {} (the script has no getup/control hand-back; state {} when last seen)", f.def.name, plural(f.ikemen.rescued), f.state_no),
        ));
    }
}

fn plural(count: u32) -> String {
    format!("{count} recovery(ies) forced by the watchdog")
}

fn check_console(m: &Match, label: &str, issues: &mut Vec<Issue>) {    let lines: Vec<&String> = m.console.lines().filter(|l| l.starts_with("warning: ")).collect();
    if lines.is_empty() {
        return;
    }
    let mut unique: BTreeSet<&String> = BTreeSet::new();
    for line in lines {
        unique.insert(line);
    }
    for line in unique.into_iter().take(4) {
        issues.push(Issue::new(Severity::Minor, "mechanics", "fight-engine-warning", label, line.clone()));
    }
}
