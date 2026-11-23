use super::{Phase, StoryRun, STAGES};
use crate::i18n::tr3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Speaker {
    Boss,
    Hero,
}

pub const PAGES: usize = 4;

pub fn pending(run: &StoryRun) -> bool {
    run.phase == Phase::Intro
        && run.stage + 1 == STAGES.len()
        && run.is_boss_fight()
        && run.prefight < PAGES
}

pub fn line(page: usize) -> (Speaker, &'static str) {
    match page {
        0 => (
            Speaker::Boss,
            tr3(
                "Cinco andares de capangas, {hero}. E você ainda respira.",
                "Five floors of thugs, {hero}. And you are still breathing.",
                "Cinco pisos de matones, {hero}. Y sigues respirando.",
            ),
        ),
        1 => (
            Speaker::Hero,
            tr3(
                "Cada andar foi mais um motivo para chegar aqui, {boss}.",
                "Every floor was one more reason to reach you, {boss}.",
                "Cada piso fue un motivo más para llegar hasta ti, {boss}.",
            ),
        ),
        2 => (
            Speaker::Boss,
            tr3(
                "Esta cidade é minha. Você é só barulho na minha cobertura.",
                "This city is mine. You are just noise on my rooftop.",
                "Esta ciudad es mía. Solo eres ruido en mi azotea.",
            ),
        ),
        _ => (
            Speaker::Hero,
            tr3(
                "Então vamos ver quem faz mais barulho.",
                "Then let's see who makes more noise.",
                "Entonces veamos quién hace más ruido.",
            ),
        ),
    }
}

pub fn speaker_name<'a>(speaker: Speaker, hero: &'a str, boss: &'a str) -> &'a str {
    match speaker {
        Speaker::Boss => boss,
        Speaker::Hero => hero,
    }
}

pub fn fill(text: &str, hero: &str, boss: &str) -> String {
    text.replace("{hero}", hero).replace("{boss}", boss)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dialogue_gates_only_the_final_boss_intro_and_ends() {
        let mut run = StoryRun::default();
        run.stage = STAGES.len() - 1;
        assert!(!pending(&run));
        run.wave = run.stage_def().waves.len();
        assert!(pending(&run));
        run.prefight = PAGES;
        assert!(!pending(&run));
    }

    #[test]
    fn names_fill_every_slot() {
        for page in 0..PAGES {
            let text = fill(line(page).1, "Kaito", "Valdric");
            assert!(!text.contains('{'));
        }
    }
}
