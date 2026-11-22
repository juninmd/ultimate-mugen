use crate::chardef::{MoveInfo, StateDef};
use crate::commands::CommandDef;
use std::collections::HashMap;

fn is_hold_or_recovery(name: &str) -> bool {
    let b = name.as_bytes();
    (b.len() >= 4 && b[..4].eq_ignore_ascii_case(b"hold")) || (b.len() >= 8 && b[..8].eq_ignore_ascii_case(b"recovery"))
}

pub(super) fn movelist(commands: &[CommandDef], states: &HashMap<i32, StateDef>) -> Vec<MoveInfo> {
    let mut movelist: Vec<MoveInfo> = Vec::with_capacity(commands.len().min(64));
    for c in commands.iter().filter(|c| !c.raw.is_empty()) {
        if !c.raw.contains(',') && !c.raw.contains('+') {
            continue;
        }
        if is_hold_or_recovery(&c.name) || movelist.iter().any(|m| m.command == c.name) {
            continue;
        }
        movelist.push(MoveInfo { name: c.name.replace('_', " "), input: c.raw.clone(), command: c.name.clone(), cost: crate::move_cost::cost_of(states, &c.name), drain: false, forms: u32::MAX, state: -1 });
    }
    movelist
}

pub(super) fn ai_commands(commands: &[CommandDef]) -> Vec<String> {
    commands
        .iter()
        .filter(|c| {
            let is_dir = ["fwd", "back", "up", "down", "ff", "bb"].iter().any(|d| d.eq_ignore_ascii_case(&c.name));
            !is_hold_or_recovery(&c.name) && !is_dir
        })
        .map(|c| c.name.clone())
        .collect()
}
