use super::{Phase, StoryRun, STAGES};
use crate::i18n::tr3;
use ume_native::LocalText;

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

pub fn ally_page(name: &str) -> LocalText {
    let line = |text: &str| text.replace("{name}", name);
    LocalText(
        [
            ("en", "Dawn found {name} still standing at your side. Some victories are never won alone."),
            ("pt", "O amanhecer encontrou {name} ainda de pé ao seu lado. Algumas vitórias nunca são sozinhas."),
            ("es", "El amanecer encontró a {name} aún en pie a tu lado. Algunas victorias nunca se logran solo."),
        ]
        .into_iter()
        .map(|(lang, text)| (lang.to_string(), line(text)))
        .collect(),
    )
}

pub fn joins(name: &str) -> String {
    format!(
        "{} {}",
        name.to_uppercase(),
        tr3("ENTRA NA LUTA", "JOINS THE FIGHT", "SE UNE A LA LUCHA")
    )
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
        "Esq/Dir ou toque nas laterais; confirme ou toque no nome",
        "Left/right or tap sides; confirm or tap the name",
        "Izq/Der o toca los lados; confirma o toca el nombre",
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
    fn ally_page_names_the_helper_in_every_language() {
        let page = ally_page("Shion");
        for lang in ["en", "pt", "es"] {
            assert!(page.pick(lang).contains("Shion"));
        }
    }

    #[test]
    fn joins_line_names_the_helper() {
        assert!(joins("Shion").contains("SHION"));
    }

    #[test]
    fn cursor_wraps_over_the_none_slot_and_every_candidate() {
        assert_eq!(step(0, 3, false), 3);
        assert_eq!(step(3, 3, true), 0);
        assert_eq!(step(1, 3, true), 2);
        assert_eq!(step(0, 0, true), 0);
    }
}
