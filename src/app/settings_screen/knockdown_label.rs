use crate::i18n::tr3;
use ume_engine::matchs::{KnockdownScope, KNOCKDOWN_PRESETS};

pub(super) fn knockdown_label(index: usize) -> String {
    let rule = KNOCKDOWN_PRESETS[index.min(KNOCKDOWN_PRESETS.len() - 1)];
    let scope = match rule.scope {
        KnockdownScope::Off => return tr3("Clássico", "Classic", "Clásico").to_string(),
        KnockdownScope::Native => tr3("Nativos", "Native", "Nativos"),
        KnockdownScope::All => tr3("Todos", "Everyone", "Todos"),
    };
    format!("{scope} · {:.0}%", rule.threshold_percent)
}
