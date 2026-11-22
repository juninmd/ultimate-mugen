use crate::i18n::tr3;

pub fn action_label(index: usize) -> &'static str {
    match index {
        0 => tr3("Cima", "Up", "Arriba"),
        1 => tr3("Baixo", "Down", "Abajo"),
        2 => tr3("Esquerda", "Left", "Izquierda"),
        3 => tr3("Direita", "Right", "Derecha"),
        4 => tr3("Soco fraco (LP)", "Light punch (LP)", "Puñetazo débil (LP)"),
        5 => tr3(
            "Soco forte (HP)",
            "Heavy punch (HP)",
            "Puñetazo fuerte (HP)",
        ),
        6 => tr3("Soco 3", "Punch 3", "Puñetazo 3"),
        7 => tr3("Chute fraco (LK)", "Light kick (LK)", "Patada débil (LK)"),
        8 => tr3("Chute forte (HK)", "Heavy kick (HK)", "Patada fuerte (HK)"),
        9 => tr3("Chute 3", "Kick 3", "Patada 3"),
        10 => tr3("Especial 1", "Special 1", "Especial 1"),
        11 => tr3("Especial 2", "Special 2", "Especial 2"),
        12 => tr3("Especial 3", "Special 3", "Especial 3"),
        13 => tr3("Despertar", "Awaken", "Despertar"),
        14 => tr3("Troca (Tag)", "Switch (Tag)", "Cambio (Tag)"),
        15 => tr3(
            "Carregar ki (Start)",
            "Charge ki (Start)",
            "Cargar ki (Start)",
        ),
        16 => tr3(
            "Andar p/ o fundo (Story)",
            "Step back in (Story)",
            "Ir al fondo (Story)",
        ),
        17 => tr3(
            "Andar p/ a frente (Story)",
            "Step forward out (Story)",
            "Ir al frente (Story)",
        ),
        _ => tr3("Defender (Story)", "Guard (Story)", "Defender (Story)"),
    }
}

pub fn key_label(raw: &'static str) -> &'static str {
    match raw {
        "Seta Cima" => tr3(raw, "Up Arrow", "Flecha Arriba"),
        "Seta Baixo" => tr3(raw, "Down Arrow", "Flecha Abajo"),
        "Seta Esq." => tr3(raw, "Left Arrow", "Flecha Izq."),
        "Seta Dir." => tr3(raw, "Right Arrow", "Flecha Der."),
        "Espaço" => tr3(raw, "Space", "Espacio"),
        "Shift Esq." => tr3(raw, "Left Shift", "Shift Izq."),
        "Shift Dir." => tr3(raw, "Right Shift", "Shift Der."),
        "Ctrl Esq." => tr3(raw, "Left Ctrl", "Ctrl Izq."),
        "Ctrl Dir." => tr3(raw, "Right Ctrl", "Ctrl Der."),
        "Alt Esq." => tr3(raw, "Left Alt", "Alt Izq."),
        "Alt Dir." => tr3(raw, "Right Alt", "Alt Der."),
        other => other,
    }
}

pub fn move_keys(ids: [&str; 4]) -> String {
    if ids == ["Up", "Down", "Left", "Right"] {
        tr3("Setas", "Arrow keys", "Flechas").to_string()
    } else {
        ids.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_action_has_its_own_label() {
        let labels: std::collections::BTreeSet<&str> =
            (0..super::super::ACTIONS.len()).map(action_label).collect();
        assert_eq!(labels.len(), super::super::ACTIONS.len());
    }

    #[test]
    fn custom_move_keys_are_listed() {
        assert_eq!(move_keys(["W", "S", "A", "D"]), "W S A D");
    }

    #[test]
    fn plain_keys_pass_through() {
        assert_eq!(key_label("Q"), "Q");
        assert_eq!(key_label("Num 4"), "Num 4");
    }
}
