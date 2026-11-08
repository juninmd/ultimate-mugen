use crate::i18n::tr;

pub fn lookup(key: &str) -> Option<&'static str> {
    match key {
        "menu.story" => Some(menu_title()),
        "desc.story" => Some(menu_description()),
        _ => None,
    }
}

pub fn menu_title() -> &'static str {
    tr("História", "Story")
}

pub fn menu_description() -> &'static str {
    tr(
        "Fases de pancadaria: ondas de capangas e um chefe por fase",
        "Brawler stages: waves of thugs and a boss at the end of each",
    )
}

pub fn stage_title(stage: usize) -> &'static str {
    match stage {
        0 => tr("BECOS DA CIDADE", "BACK ALLEYS"),
        1 => tr("METRÔ", "SUBWAY"),
        2 => tr("BAMBUZAL", "BAMBOO GROVE"),
        3 => tr("ÁREA INDUSTRIAL", "INDUSTRIAL AREA"),
        _ => tr("COBERTURA DO CHEFÃO", "KINGPIN ROOFTOP"),
    }
}

pub fn stage_line(stage: usize) -> &'static str {
    match stage {
        0 => tr("A gangue domina as ruas. Abra caminho a socos.", "The gang runs the streets. Fight your way through."),
        1 => tr("Nos trilhos, ninguém ouve você gritar.", "Down on the tracks, nobody hears you shout."),
        2 => tr("Sombras se movem entre o bambu.", "Shadows move between the bamboo."),
        3 => tr("Fornalhas, aço e gigantes de aço.", "Furnaces, steel and giants made of it."),
        _ => tr("O último andar. O chefão espera.", "The top floor. The kingpin waits."),
    }
}

pub fn stage_label(stage: usize) -> String {
    format!("{} {}", tr("FASE", "STAGE"), stage + 1)
}

pub fn wave_label(wave: usize, total: usize, boss: bool) -> String {
    if boss {
        return tr("CHEFE", "BOSS").to_string();
    }
    format!("{} {}/{}", tr("ONDA", "WAVE"), wave + 1, total)
}

pub fn go() -> &'static str {
    tr("VAI!", "GO!")
}

pub fn boss_banner() -> &'static str {
    tr("CHEFE DA FASE", "STAGE BOSS")
}

pub fn press_start() -> &'static str {
    tr("Pressione OK para começar", "Press OK to begin")
}

pub fn stage_clear() -> &'static str {
    tr("FASE CONCLUÍDA", "STAGE CLEAR")
}

pub fn story_clear() -> &'static str {
    tr("HISTÓRIA CONCLUÍDA", "STORY COMPLETE")
}

pub fn continue_label() -> &'static str {
    tr("Continuar", "Continue")
}

pub fn game_over() -> &'static str {
    tr("FIM DE JOGO", "GAME OVER")
}

pub fn quit_label() -> &'static str {
    tr("Menu", "Menu")
}

pub fn score_line(score: i64) -> String {
    format!("{} {}", score, tr("PONTOS", "POINTS"))
}

pub fn credits_line(credits: u8) -> String {
    format!("{} {}", tr("CRÉDITOS", "CREDITS"), credits)
}

pub fn hud(stage: usize, wave: usize, total: usize, boss: bool, score: i64) -> String {
    format!(
        "{} · {} · {}",
        stage_label(stage),
        wave_label(wave, total, boss),
        score_line(score)
    )
}
