use crate::i18n::tr3;

pub fn fx_level(i: usize) -> &'static str {
    match i {
        0 => tr3("Desligado", "Off", "Apagado"),
        1 => tr3("Médio", "Medium", "Medio"),
        _ => tr3("Alto (bloom)", "High (bloom)", "Alto (bloom)"),
    }
}

pub fn impact_level(i: usize) -> &'static str {
    match i {
        0 => tr3("Mínimo (fotossensibilidade)", "Minimum (photosensitivity)", "Mínimo (fotosensibilidad)"),
        1 => tr3("Reduzido", "Reduced", "Reducido"),
        2 => tr3("Normal", "Normal", "Normal"),
        _ => tr3("Intenso", "Intense", "Intenso"),
    }
}

pub fn art_quality(i: usize) -> &'static str {
    match i {
        0 => tr3("Automática", "Automatic", "Automática"),
        1 => tr3("Padrão (3×)", "Standard (3×)", "Estándar (3×)"),
        2 => tr3("Alta (4,5×)", "High (4.5×)", "Alta (4,5×)"),
        _ => tr3("Ultra 4K (6×)", "Ultra 4K (6×)", "Ultra 4K (6×)"),
    }
}
