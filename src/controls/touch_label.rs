use ume_engine::input::k;

pub fn label(bit: u32) -> &'static str {
    match bit {
        k::X => crate::i18n::tr3("SOCO", "PUNCH", "PUÑO"),
        k::Y => crate::i18n::tr3("SOCO+", "PUNCH+", "PUÑO+"),
        k::Z => crate::i18n::tr3("SOCO 3", "PUNCH 3", "PUÑO 3"),
        k::A => crate::i18n::tr3("CHUTE", "KICK", "PATA"),
        k::B => crate::i18n::tr3("CHUTE+", "KICK+", "PATA+"),
        k::C => crate::i18n::tr3("CHUTE 3", "KICK 3", "PATA 3"),
        k::D2 => crate::i18n::tr3("ESPECIAL", "SPECIAL", "ESPECIAL"),
        k::W => "SUPER",
        k::T => crate::i18n::tr3("DESPERTAR", "AWAKEN", "DESPERTAR"),
        k::S => crate::i18n::tr3("CARREGAR", "CHARGE", "CARGAR"),
        _ => "II",
    }
}

pub fn button((fallback, bits): (&'static str, u32)) -> &'static str {
    match bits {
        k::X | k::Y | k::Z | k::A | k::B | k::C | k::D2 | k::T | k::S => label(bits),
        k::E => crate::i18n::tr3("ESPECIAL 3", "SPECIAL 3", "ESPECIAL 3"),
        k::TAG => crate::i18n::tr3("TROCA", "TAG", "CAMBIO"),
        k::GUARD => crate::i18n::tr3("DEFESA", "GUARD", "DEFENSA"),
        _ => fallback,
    }
}
