use super::sim::{self, Outcome};
use crate::routes::Route;
use serde::Serialize;
use std::fmt::Write as _;
use std::sync::Arc;
use ume_engine::chardef::CharDef;

#[derive(Serialize)]
pub struct AddedMove {
    pub route: Route,
    pub outcome: Outcome,
    pub reached: bool,
}

fn when_of(d: &CharDef, state: i32) -> &'static str {
    match d.states.get(&state).and_then(|s| s.param("type")).map(|p| p.raw.trim().to_ascii_uppercase()) {
        Some(t) if t == "A" => "air",
        Some(t) if t == "C" => "crouch",
        _ => "stand",
    }
}

pub fn added(base: &CharDef, full: &Arc<CharDef>) -> Vec<AddedMove> {
    full.movelist
        .iter()
        .filter(|m| !base.commands.iter().any(|c| c.name.eq_ignore_ascii_case(&m.command)))
        .map(|m| {
            let route = Route { command: m.command.clone(), input: m.input.clone(), state: m.state, when: when_of(full, m.state), cost: m.cost as i32 };
            let outcome = sim::run(full, &route).outcome;
            AddedMove { reached: outcome.states.contains(&m.state), route, outcome }
        })
        .collect()
}

pub fn markdown(moves: &[AddedMove], md: &mut String) {
    if moves.is_empty() {
        return;
    }
    *md += "\n## Added by UMF\n\nMoves that exist only in the bundle (native `moves`, `scripts`, `mugenRoutes`), played from their real input on the bundle.\n\n| command | input | when | state | reached | states visited | damage | sounds |\n|---|---|---|---|---|---|---|---|\n";
    for m in moves {
        let _ = writeln!(md, "| {} | `{}` | {} | {} | {} | {:?} | {:.0} | {} |", m.route.command, m.route.input.replace('|', "\\|"), m.route.when, m.route.state, if m.reached { "yes" } else { "no" }, m.outcome.states, m.outcome.damage, m.outcome.sounds);
    }
}
