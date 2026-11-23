use super::{Phase, StoryRun, STAGES};
use crate::i18n::tr3;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum HelperPick {
    #[default]
    Pending,
    Declined,
    Chosen(String),
}

pub fn pending(run: &StoryRun) -> bool {
    run.phase == Phase::Intro
        && run.stage + 1 == STAGES.len()
        && run.is_boss_fight()
        && run.mercy.is_some()
        && run.helper == HelperPick::Pending
}

pub fn active(run: &StoryRun) -> Option<&str> {
    match &run.helper {
        HelperPick::Chosen(id) if run.stage + 1 == STAGES.len() && run.is_boss_fight() => {
            Some(id.as_str())
        }
        _ => None,
    }
}

pub fn step(cursor: usize, candidates: usize, forward: bool) -> usize {
    let slots = candidates + 1;
    if forward {
        (cursor + 1) % slots
    } else {
        (cursor + slots - 1) % slots
    }
}

pub fn prompt() -> &'static str {
    tr3(
        "A última luta. Chame alguém para lutar ao seu lado.",
        "The final fight. Call someone to stand beside you.",
        "La última lucha. Llama a alguien para que luche a tu lado.",
    )
}

pub fn none_label() -> &'static str {
    tr3("LUTAR SOZINHO", "GO SOLO", "IR SOLO")
}

pub fn hint() -> &'static str {
    tr3(
        "Esquerda ou direita para escolher, confirmar para aceitar",
        "Left or right to choose, confirm to accept",
        "Izquierda o derecha para elegir, confirmar para aceptar",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helper_is_asked_only_after_mercy_on_the_final_boss_intro() {
        let mut run = StoryRun::default();
        run.stage = STAGES.len() - 1;
        run.wave = run.stage_def().waves.len();
        assert!(!pending(&run));
        run.mercy = Some(false);
        assert!(pending(&run));
        run.helper = HelperPick::Declined;
        assert!(!pending(&run));
    }

    #[test]
    fn chosen_helper_only_fights_the_final_boss() {
        let mut run = StoryRun::default();
        run.helper = HelperPick::Chosen("kaito".into());
        assert_eq!(active(&run), None);
        run.stage = STAGES.len() - 1;
        run.wave = run.stage_def().waves.len();
        assert_eq!(active(&run), Some("kaito"));
    }

    #[test]
    fn cursor_wraps_over_the_none_slot_and_every_candidate() {
        assert_eq!(step(0, 3, false), 3);
        assert_eq!(step(3, 3, true), 0);
        assert_eq!(step(1, 3, true), 2);
        assert_eq!(step(0, 0, true), 0);
    }
}
