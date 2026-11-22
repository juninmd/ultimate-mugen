mod added;
mod evidence;
pub(crate) mod sim;
mod structural;

use crate::routes::Route;
use serde::Serialize;
use std::fmt::Write as _;
use std::sync::Arc;
use ume_engine::chardef::CharDef;

pub use added::AddedMove;
pub use evidence::evidence;
pub use sim::{Outcome, TICKS};

#[derive(Serialize)]
pub struct MoveParity {
    pub route: Route,
    pub original: Outcome,
    pub umf: Outcome,
    pub identical: bool,
    pub divergence: Option<(usize, String, String)>,
}

#[derive(Serialize)]
pub struct SectionParity {
    pub name: &'static str,
    pub compared: usize,
    pub differ: usize,
    pub examples: Vec<String>,
}

#[derive(Serialize)]
pub struct Parity {
    pub name: String,
    pub identical: bool,
    pub sections: Vec<SectionParity>,
    pub moves: Vec<MoveParity>,
    pub added: Vec<AddedMove>,
}

pub fn parity(name: &str, orig: &Arc<CharDef>, umf: &Arc<CharDef>, full: &Arc<CharDef>, routes: &[Route]) -> Parity {
    let sections: Vec<SectionParity> = structural::structural(orig, umf)
        .into_iter()
        .map(|(name, s)| SectionParity { name, compared: s.compared, differ: s.differ.len(), examples: s.differ.into_iter().take(5).collect() })
        .collect();
    let moves: Vec<MoveParity> = routes
        .iter()
        .map(|route| {
            let (a, b) = (sim::run(orig, route), sim::run(umf, route));
            let divergence = sim::first_divergence(&a, &b);
            MoveParity { route: route.clone(), identical: divergence.is_none(), original: a.outcome, umf: b.outcome, divergence }
        })
        .collect();
    let identical = sections.iter().all(|s| s.differ == 0) && moves.iter().all(|m| m.identical);
    Parity { name: name.into(), identical, sections, moves, added: added::added(umf, full) }
}

impl Parity {
    pub fn summary(&self) -> String {
        let same = self.moves.iter().filter(|m| m.identical).count();
        format!("{}: {} | data sections identical {}/{} | moves identical {same}/{}", self.name, if self.identical { "IDENTICAL" } else { "DIFFERS" }, self.sections.iter().filter(|s| s.differ == 0).count(), self.sections.len(), self.moves.len())
    }

    pub fn markdown(&self) -> String {
        let mut md = format!("# Parity report: {}\n\n**{}**\n\nEvery move is played from its real input on the original M.U.G.E.N fighter and on the UMF bundle, {TICKS} ticks each, and every tick of both fighters (state, animation, frame, position, velocity, control, life, power, helpers, sounds) is compared.\n\n## Data\n\n| section | compared | different | examples |\n|---|---|---|---|\n", self.name, self.summary());
        for s in &self.sections {
            let _ = writeln!(md, "| {} | {} | {} | {} |", s.name, s.compared, s.differ, s.examples.join("; ").replace('|', "\\|"));
        }
        md += "\n## Moves\n\n| command | input | when | state | reached | states visited | damage | sounds | result |\n|---|---|---|---|---|---|---|---|---|\n";
        for m in &self.moves {
            let result = match &m.divergence {
                None => "identical".to_string(),
                Some((t, _, _)) => format!("diverges at tick {t}"),
            };
            let reached = if m.original.states.contains(&m.route.state) { "yes" } else { "no" };
            let _ = writeln!(md, "| {} | `{}` | {} | {} | {reached} | {:?} | {:.0} | {} | {result} |", m.route.command, m.route.input.replace('|', "\\|"), m.route.when, m.route.state, m.original.states, m.original.damage, m.original.sounds);
        }
        for m in self.moves.iter().filter(|m| !m.identical) {
            if let Some((t, a, b)) = &m.divergence {
                let _ = write!(md, "\n### {} (tick {t})\n\n```\noriginal {a}\numf      {b}\n```\n", m.route.command);
            }
        }
        added::markdown(&self.added, &mut md);
        md
    }
}
