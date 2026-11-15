mod support;
use std::sync::Arc;
use support::*;
use ume_engine::chardef::CharDef;
use ume_engine::commands::CommandDef;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::matchs::Match;
use ume_engine::native::{compile_native, NativeChar};

const ULTIMATE_COST: f32 = 3000.0;
const FULL_METER: f32 = 3000.0;
const MINIMUM_DAMAGE: f32 = 380.0;
const SUPER_MARGIN: f32 = 1.2;
const GAPS: [f32; 6] = [20.0, 40.0, 70.0, 110.0, 160.0, 240.0];
const WATCH_TICKS: usize = 360;
const HOLD_TICKS: usize = 3;
const ROSTER_FLOOR: usize = 21;

fn characters() -> Vec<NativeChar> {
    let mut out: Vec<NativeChar> = std::fs::read_dir("../../assets/chars")
        .unwrap()
        .filter_map(|entry| {
            let path = entry.unwrap().path();
            let stem = path.file_stem()?.to_string_lossy().to_string();
            (path.extension()? == "json" && stem != "index")
                .then(|| serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap())
        })
        .collect();
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}

fn arena(def: &Arc<CharDef>) -> Match {
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(def.clone(), 0, 0, 1);
    m.add_player(def.clone(), 1, 0, 2);
    m.start_round();
    while m.round_state != 2 {
        m.tick(&[0, 0]);
    }
    m
}

fn presses(command: &CommandDef) -> Vec<u32> {
    let mut frames = Vec::new();
    for step in &command.steps {
        let bits = step.dir | step.btns;
        if step.release {
            frames.extend(std::iter::repeat_n(bits, HOLD_TICKS));
            frames.extend([0, 0]);
        } else {
            let hold = if step.charge > 0 {
                step.charge as usize + 2
            } else if step.hold {
                8
            } else {
                HOLD_TICKS
            };
            frames.extend(std::iter::repeat_n(bits, hold));
            frames.push(0);
        }
    }
    frames
}

struct Probe {
    gap: f32,
    damage: f32,
    hits: usize,
    drained: f32,
    entered: bool,
}

fn run_move(base: &Match, command: &CommandDef, state: i32, gap: f32) -> Probe {
    let mut m = base.clone();
    face_off(&mut m, gap);
    m.fighters[0].power = FULL_METER;
    let life = m.fighters[1].life;
    let mut probe = Probe {
        gap,
        damage: 0.0,
        hits: 0,
        drained: 0.0,
        entered: false,
    };
    let mut last = life;
    for bits in presses(command)
        .iter()
        .copied()
        .chain(std::iter::repeat_n(0, WATCH_TICKS))
    {
        m.tick(&[bits, 0]);
        if m.fighters[0].state_no == state {
            probe.entered = true;
            probe.drained = probe.drained.max(FULL_METER - m.fighters[0].power);
        }
        if m.fighters[1].life < last {
            probe.hits += 1;
            last = m.fighters[1].life;
        }
    }
    probe.damage = life - m.fighters[1].life;
    probe
}

fn command_of<'a>(def: &'a CharDef, id: &str) -> &'a CommandDef {
    def.commands
        .iter()
        .find(|c| c.name == format!("m_{id}"))
        .expect("move command compiled")
}

fn state_of(def: &CharDef, name: &str) -> i32 {
    def.movelist
        .iter()
        .find(|m| m.name == name)
        .map(|m| m.state)
        .expect("move state compiled")
}

#[test]
fn every_native_fighter_has_one_three_bar_ultimate() {
    let roster = characters();
    assert!(
        roster.len() >= ROSTER_FLOOR,
        "roster scan found {} fighters",
        roster.len()
    );
    let mut failures = vec![];
    for nc in &roster {
        let ults: Vec<_> = nc
            .moves
            .iter()
            .filter(|m| m.id.starts_with("ult_"))
            .collect();
        if ults.len() != 1 {
            failures.push(format!("{} has {} ultimate moves", nc.id, ults.len()));
            continue;
        }
        let ult = ults[0];
        if ult.kind != "super" {
            failures.push(format!("{} ultimate kind is {}", nc.id, ult.kind));
        }
        if ult.cost != Some(ULTIMATE_COST) {
            failures.push(format!("{} ultimate costs {:?}", nc.id, ult.cost));
        }
        if !ult.input.contains('+') {
            failures.push(format!(
                "{} ultimate needs a button combination: {}",
                nc.id, ult.input
            ));
        }
        let def = compile_native(nc, 1, false);
        match def.movelist.iter().find(|m| m.name == ult.name) {
            Some(e) if e.cost == ULTIMATE_COST => {}
            Some(e) => failures.push(format!("{} ultimate moves list cost {}", nc.id, e.cost)),
            None => failures.push(format!("{} ultimate missing from the move list", nc.id)),
        }
        if def.constants.get("data.power").copied().unwrap_or(0.0) < ULTIMATE_COST as f64 {
            failures.push(format!("{} meter cannot hold three bars", nc.id));
        }
    }
    assert!(
        failures.is_empty(),
        "ultimate roster failures:\n{}",
        failures.join("\n")
    );
}

#[test]
fn every_ultimate_drains_three_bars_and_hits_harder_than_every_super() {
    let mut failures = vec![];
    for nc in characters() {
        let Some(ult) = nc.moves.iter().find(|m| m.id.starts_with("ult_")) else {
            continue;
        };
        let def = Arc::new(compile_native(&nc, 1, true));
        let base = arena(&def);
        let best = |id: &str, name: &str| -> Probe {
            let (command, state) = (command_of(&def, id), state_of(&def, name));
            GAPS.iter()
                .map(|gap| run_move(&base, command, state, *gap))
                .max_by(|a, b| a.damage.total_cmp(&b.damage))
                .expect("probes")
        };
        let top = best(&ult.id, &ult.name);
        let super_damage = nc
            .moves
            .iter()
            .filter(|m| m.kind == "super" && !m.id.starts_with("ult_"))
            .map(|m| best(&m.id, &m.name).damage)
            .fold(0.0f32, f32::max);
        let near: Vec<i32> = GAPS
            .iter()
            .map(|gap| {
                run_move(
                    &base,
                    command_of(&def, &ult.id),
                    state_of(&def, &ult.name),
                    *gap,
                )
                .damage as i32
            })
            .collect();
        println!("{:12} {:42} dmg {:6.0} hits {:2} gap {:3.0} power {:4.0} best super {:6.0} by gap {:?}", nc.id, ult.name, top.damage, top.hits, top.gap, top.drained, super_damage, near);
        if !top.entered {
            failures.push(format!("{} ultimate never starts from its command", nc.id));
        }
        if top.drained < ULTIMATE_COST - 1.0 {
            failures.push(format!("{} ultimate drains {} power", nc.id, top.drained));
        }
        if top.hits == 0 {
            failures.push(format!("{} ultimate never connects", nc.id));
        }
        if top.damage < MINIMUM_DAMAGE {
            failures.push(format!("{} ultimate only deals {:.0}", nc.id, top.damage));
        }
        if top.damage < super_damage * SUPER_MARGIN {
            failures.push(format!(
                "{} ultimate {:.0} is not above its supers {:.0}",
                nc.id, top.damage, super_damage
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "ultimate behaviour failures:\n{}",
        failures.join("\n")
    );
}
