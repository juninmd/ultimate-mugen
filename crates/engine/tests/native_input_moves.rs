mod support;
use std::collections::BTreeSet;
use std::sync::Arc;
use support::*;
use ume_engine::chardef::CharDef;
use ume_engine::commands::CommandDef;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::matchs::Match;
use ume_engine::native::{compile_native, NativeChar};

const GAPS: [f32; 4] = [20.0, 60.0, 110.0, 220.0];
#[derive(Clone, Copy)]
enum Stance { Standing, Crouching, Airborne }
const STANCES: [Stance; 3] = [Stance::Standing, Stance::Crouching, Stance::Airborne];
const JUMP_STATE: i32 = 40;
const CROUCH_HOLD: u32 = 2;
const HOLD_TICKS: usize = 3;
const HELD_STEP_TICKS: usize = 8;
const WATCH_TICKS: usize = 45;
const FIRST_ABILITY: i32 = 200;
const FIRST_COMMON: i32 = 5000;
const VICTIM_STATES: std::ops::Range<i32> = 3800..3900;

fn is_ability_state(state: i32) -> bool { (FIRST_ABILITY..FIRST_COMMON).contains(&state) && !VICTIM_STATES.contains(&state) }

fn characters() -> Vec<NativeChar> {
    let mut out: Vec<NativeChar> = std::fs::read_dir("../../assets/chars").unwrap().filter_map(|entry| {
        let path = entry.unwrap().path();
        let stem = path.file_stem()?.to_string_lossy().to_string();
        (path.extension()? == "json" && stem != "index").then(|| serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap())
    }).collect();
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}

fn arena(def: &Arc<CharDef>) -> Match {
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(def.clone(), 0, 0, 1);
    m.add_player(def.clone(), 1, 0, 2);
    m.start_round();
    while m.round_state != 2 { m.tick(&[0, 0]); }
    m
}

fn presses(command: &CommandDef) -> Vec<u32> {
    let mut frames = Vec::new();
    for step in &command.steps {
        let bits = step.dir | step.btns;
        let hold = if step.charge > 0 { step.charge as usize + 2 } else if step.hold { HELD_STEP_TICKS } else { HOLD_TICKS };
        if step.release {
            frames.extend(std::iter::repeat_n(bits, hold));
            frames.extend([0, 0]);
        } else {
            frames.extend(std::iter::repeat_n(bits, HOLD_TICKS));
            frames.push(0);
        }
    }
    frames
}

fn abilities_reached(base: &Match, command: &CommandDef, forms: usize) -> BTreeSet<i32> {
    let mut reached = BTreeSet::new();
    let frames = presses(command);
    for gap in GAPS {
        for (stance, form) in STANCES.iter().flat_map(|stance| (0..=forms).map(move |form| (*stance, form))) {
            let mut m = base.clone();
            m.fighters[0].form = form as u8;
            face_off(&mut m, gap);
            m.fighters[0].power = 3000.0;
            let held = match stance {
                Stance::Standing => 0,
                Stance::Crouching => { for _ in 0..8 { m.tick(&[CROUCH_HOLD, 0]); } CROUCH_HOLD }
                Stance::Airborne => {
                    enter(&mut m, 0, JUMP_STATE);
                    for _ in 0..40 { if m.fighters[0].state_type == 'A' && m.fighters[0].ctrl { break; } m.tick(&[0, 0]); }
                    0
                }
            };
            for bits in frames.iter().copied().chain(std::iter::repeat_n(0, WATCH_TICKS)) {
                m.tick(&[bits | held, 0]);
                let state = m.fighters[0].state_no;
                if is_ability_state(state) { reached.insert(state); }
            }
        }
    }
    reached
}

#[test]
fn every_command_of_every_fighter_performs_its_move_from_real_inputs() {
    let (mut dead_commands, mut unreachable_moves) = (Vec::new(), Vec::new());
    for nc in characters() {
        let def = Arc::new(compile_native(&nc, 1, true));
        let base = arena(&def);
        let mut from_inputs = BTreeSet::new();
        let mut tested = 0;
        for command in def.commands.iter().filter(|c| c.steps.iter().any(|s| s.btns != 0)) {
            let reached = abilities_reached(&base, command, def.forms.len());
            tested += 1;
            if reached.is_empty() { dead_commands.push(format!("{} command {} ({}) triggered no ability state", nc.id, command.name, command.raw)); }
            from_inputs.extend(reached);
        }
        let declared: BTreeSet<i32> = def.states.keys().copied().filter(|n| is_ability_state(*n)).collect();
        let missed: Vec<i32> = declared.difference(&from_inputs).copied().collect();
        println!("{:8} {tested:2} button commands, {:2}/{:2} ability states reached from inputs, not reached {missed:?}", nc.id, declared.len() - missed.len(), declared.len());
        if !missed.is_empty() { unreachable_moves.push(format!("{} states {missed:?} cannot be started from any command", nc.id)); }
    }
    assert!(dead_commands.is_empty() && unreachable_moves.is_empty(), "dead commands:\n{}\nunreachable moves:\n{}", dead_commands.join("\n"), unreachable_moves.join("\n"));
}
