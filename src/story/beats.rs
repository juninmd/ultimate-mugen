use crate::i18n::tr3;

pub const BEATS_PER_STAGE: usize = 4;
const ALLY_BEAT: usize = 3;
pub const BEAT_SECONDS: f32 = 2.4;
pub const BEAT_FADE: f32 = 0.35;

pub fn stage_beat(stage: usize, beat: usize) -> &'static str {
    let scripted = super::script::active().stage_beat(stage, beat, crate::i18n::get());
    if !scripted.is_empty() {
        return scripted;
    }
    let beat = beat.min(BEATS_PER_STAGE - 1);
    if beat == ALLY_BEAT {
        return ally_line(stage);
    }
    match (stage, beat) {
        (0, 0) => tr3(
            "A gangue domina as ruas. Abra caminho a socos.",
            "The gang runs the streets. Fight your way through.",
            "La banda domina las calles. Abre camino a golpes.",
        ),
        (0, 1) => tr3(
            "Ontem à noite levaram o seu bairro. Pegue de volta.",
            "They took your block last night. Take it back.",
            "Anoche se llevaron tu barrio. Recupéralo.",
        ),
        (0, _) => tr3(
            "Cada beco tem um vigia. E todo vigia fala.",
            "Every alley has a lookout. Every lookout talks.",
            "Cada callejón tiene un vigía. Y todo vigía habla.",
        ),
        (1, 0) => tr3(
            "Nos trilhos, ninguém ouve você gritar.",
            "Down on the tracks, nobody hears you shout.",
            "En las vías, nadie oye tus gritos.",
        ),
        (1, 1) => tr3(
            "O metrô é a última rota segura deles.",
            "The metro is their last safe route.",
            "El metro es su última ruta segura.",
        ),
        (1, _) => tr3(
            "Não olhe para trás. Alguém está seguindo.",
            "Don't look back. Something is following.",
            "No mires atrás. Alguien te sigue.",
        ),
        (2, 0) => tr3(
            "Sombras se movem entre o bambu.",
            "Shadows move between the bamboo.",
            "Las sombras se mueven entre el bambú.",
        ),
        (2, 1) => tr3(
            "Alguém aqui sabe o seu nome.",
            "Someone here knows your name.",
            "Alguien aquí sabe tu nombre.",
        ),
        (2, _) => tr3(
            "O lobo rosna. Ele já viu isso antes.",
            "The wolf growls. It has seen this before.",
            "El lobo gruñe. Ya lo ha visto antes.",
        ),
        (3, 0) => tr3(
            "Fornalhas, aço e gigantes de aço.",
            "Furnaces, steel and giants made of it.",
            "Hornos, acero y gigantes de acero.",
        ),
        (3, 1) => tr3(
            "O chefe construiu este lugar com dívidas.",
            "Their boss built this place on debt.",
            "Su jefe levantó este lugar con deudas.",
        ),
        (3, _) => tr3(
            "Cada martelada é uma contagem regressiva.",
            "Every hammer blow is a countdown.",
            "Cada martillazo es una cuenta regresiva.",
        ),
        (_, 0) => tr3(
            "O último andar. O chefão espera.",
            "The top floor. The kingpin waits.",
            "La última planta. El jefe espera.",
        ),
        (_, 1) => tr3(
            "É aqui que se decide quem manda na cidade.",
            "This is where the city's rule is decided.",
            "Aquí se decide quién manda en la ciudad.",
        ),
        (_, _) => tr3(
            "Vença, e as ruas são suas. Perca, e esquecem você.",
            "Win, and the streets are yours. Lose, and they forget you.",
            "Gana, y las calles son tuyas. Pierde, y te olvidarán.",
        ),
    }
}

pub fn ally_line(stage: usize) -> &'static str {
    match stage {
        0 => tr3(
            "Lobo: Ele cheira a medo. Fique perto de mim.",
            "Wolf: He smells of fear. Stay close.",
            "Lobo: Huele a miedo. Quédate cerca.",
        ),
        1 => tr3(
            "Lobo: Os trens não param. Nós também não.",
            "Wolf: The trains don't stop. Neither do we.",
            "Lobo: Los trenes no paran. Nosotros tampoco.",
        ),
        2 => tr3(
            "Lobo: Essa sombra tem o seu cheiro.",
            "Wolf: That shadow carries your scent.",
            "Lobo: Esa sombra tiene tu olor.",
        ),
        3 => tr3(
            "Lobo: Cuidado com o aço. Ele não sente dor.",
            "Wolf: Watch the steel. It feels no pain.",
            "Lobo: Cuidado con el acero. No siente dolor.",
        ),
        _ => tr3(
            "Lobo: Ele não vai cair sozinho. Vamos.",
            "Wolf: He won't fall alone. Let's go.",
            "Lobo: No caerá solo. Vamos.",
        ),
    }
}

pub fn beat_index(t: f32) -> usize {
    ((t / BEAT_SECONDS) as usize).min(BEATS_PER_STAGE - 1)
}

pub fn beat_alpha(t: f32) -> f32 {
    let local = t - beat_index(t) as f32 * BEAT_SECONDS;
    (local / BEAT_FADE).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_stage_has_three_distinct_beats() {
        for stage in 0..5 {
            let lines: Vec<&str> = (0..BEATS_PER_STAGE).map(|b| stage_beat(stage, b)).collect();
            assert!(lines.iter().all(|l| !l.is_empty()));
            assert_ne!(lines[0], lines[1]);
            assert_ne!(lines[1], lines[2]);
            assert_ne!(lines[2], lines[3]);
            assert!(lines[ALLY_BEAT].starts_with("Lobo:") || lines[ALLY_BEAT].starts_with("Wolf:"));
        }
    }

    #[test]
    fn beats_advance_with_time_and_stop_at_the_last() {
        assert_eq!(beat_index(0.0), 0);
        assert_eq!(beat_index(BEAT_SECONDS), 1);
        assert_eq!(beat_index(BEAT_SECONDS * 10.0), BEATS_PER_STAGE - 1);
    }

    #[test]
    fn each_beat_fades_in_from_its_start() {
        assert_eq!(beat_alpha(0.0), 0.0);
        assert!((beat_alpha(BEAT_SECONDS + BEAT_FADE) - 1.0).abs() < 1e-4);
    }
}
