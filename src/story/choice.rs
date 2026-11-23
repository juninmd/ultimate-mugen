use super::{Phase, StoryRun, STAGES};
use crate::i18n::tr3;

pub fn pending(run: &StoryRun) -> bool {
    run.phase == Phase::Intro
        && run.stage + 1 == STAGES.len()
        && run.is_boss_fight()
        && run.mercy.is_none()
}

pub fn prompt() -> &'static str {
    tr3(
        "Ele está de joelhos. Você decide o que acontece.",
        "He is on his knees. Your call.",
        "Está de rodillas. Tú decides.",
    )
}

pub fn spare_label() -> &'static str {
    tr3("POUPAR", "SPARE", "PERDONAR")
}

pub fn finish_label() -> &'static str {
    tr3("DERRUBAR", "FINISH", "DERRIBAR")
}

pub fn hint() -> &'static str {
    tr3(
        "Use esquerda/direita ou toque em uma opção",
        "Use left/right or tap an option",
        "Usa izquierda/derecha o toca una opción",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_final_boss_intro_asks_before_the_fight() {
        let mut run = StoryRun::default();
        assert!(!pending(&run));
        run.stage = STAGES.len() - 1;
        assert!(!pending(&run));
        run.wave = run.stage_def().waves.len();
        assert!(pending(&run));
        run.mercy = Some(false);
        assert!(!pending(&run));
    }

    #[test]
    fn mercy_swaps_the_epilogue_and_the_default_is_kept() {
        use crate::story::epilogue::of_with;
        let spared = of_with("kaito", Some(true)).expect("mercy epilogue");
        let normal = of_with("kaito", Some(false)).expect("default epilogue");
        assert_ne!(spared.beats("en"), normal.beats("en"));
        assert_eq!(
            of_with("kaito", None).unwrap().beats("en"),
            normal.beats("en")
        );
    }
}
