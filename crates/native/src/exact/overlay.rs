use super::super::compile_moves::compile_moves;
use super::super::throw::OVERLAY_BLOCK;
use super::super::types::NativeChar;
use std::collections::HashMap;
use ume_engine::chardef::CharDef;
use ume_engine::cns::parse_states;

pub fn overlay_block(d: &CharDef) -> i32 {
    let top = d.states.keys().copied().max().unwrap_or(0).max(0);
    (top / OVERLAY_BLOCK + 1) * OVERLAY_BLOCK
}

pub fn overlay_moves(nc: &NativeChar, anim_no: &HashMap<String, i32>, d: &mut CharDef) -> HashMap<String, i32> {
    if nc.moves.is_empty() {
        return HashMap::new();
    }
    let block = overlay_block(d);
    let (mut minus1, mut cns, mut commands) = (String::from("[Statedef -1]\n\n"), String::new(), vec![]);
    let (movelist, move_states) = compile_moves(nc, anim_no, block, &mut commands, &mut minus1, &mut cns);
    let mut fresh = HashMap::new();
    parse_states(&(minus1 + &cns), &mut fresh, true);
    if let Some(mut routes) = fresh.remove(&-1) {
        match d.states.get_mut(&-1) {
            Some(existing) => {
                routes.controllers.append(&mut existing.controllers);
                existing.controllers = routes.controllers;
            }
            None => {
                d.states.insert(-1, routes);
            }
        }
    }
    for (id, state) in fresh {
        d.states.entry(id).or_insert(state);
    }
    d.ai_commands.extend(movelist.iter().map(|m| m.command.clone()));
    d.commands.extend(commands);
    d.movelist.extend(movelist);
    move_states
}
