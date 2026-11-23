use crate::i18n::tr3;

pub fn of(stage: usize) -> &'static str {
    match stage {
        0 => tr3(
            "Dica: quebre caixas e barris, a comida recupera vida.",
            "Tip: smash crates and barrels, food restores life.",
            "Consejo: rompe cajas y barriles, la comida recupera vida.",
        ),
        1 => tr3(
            "Dica: avance só quando aparecer GO, os inimigos esperam na tela travada.",
            "Tip: move on when GO appears, foes wait in the locked screen.",
            "Consejo: avanza cuando aparezca GO, los enemigos esperan en la pantalla fija.",
        ),
        2 => tr3(
            "Dica: continuar mantém sua pontuação, mas custa nota no final.",
            "Tip: continuing keeps your score but costs you rank at the end.",
            "Consejo: continuar conserva tu puntuación, pero cuesta rango al final.",
        ),
        3 => tr3(
            "Dica: sua vida passa de uma luta para a outra, só uma parte volta entre as ondas.",
            "Tip: your life carries from fight to fight, only part of it returns between waves.",
            "Consejo: tu vida pasa de una lucha a otra, solo parte vuelve entre oleadas.",
        ),
        _ => tr3(
            "Dica: na luta final, chame um ajudante e pegue o chefe em duas frentes.",
            "Tip: in the final fight, call a helper and hit the boss from two sides.",
            "Consejo: en la lucha final, llama a un ayudante y ataca al jefe por dos frentes.",
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_stage_has_its_own_tip() {
        let tips: Vec<&str> = (0..5).map(of).collect();
        for (i, a) in tips.iter().enumerate() {
            assert!(!a.is_empty());
            assert!(tips[i + 1..].iter().all(|b| a != b));
        }
    }
}
