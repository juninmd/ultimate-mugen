use crate::i18n::tr3;

const THRESHOLDS: [u32; 7] = [0, 40, 120, 300, 650, 1200, 2500];

pub(crate) struct Rank {
    pub index: usize,
    pub progress: f32,
    pub next: Option<u32>,
}

pub(crate) fn rank(xp: u32) -> Rank {
    let index = THRESHOLDS.iter().rposition(|&t| xp >= t).unwrap_or(0);
    let floor = THRESHOLDS[index];
    match THRESHOLDS.get(index + 1) {
        Some(&next) => Rank {
            index,
            progress: (xp - floor) as f32 / (next - floor) as f32,
            next: Some(next),
        },
        None => Rank {
            index,
            progress: 1.0,
            next: None,
        },
    }
}

pub(crate) fn rank_name(index: usize) -> &'static str {
    match index {
        0 => tr3("Novato", "Rookie", "Novato"),
        1 => tr3("Aprendiz", "Apprentice", "Aprendiz"),
        2 => tr3("Lutador", "Fighter", "Luchador"),
        3 => tr3("Veterano", "Veteran", "Veterano"),
        4 => tr3("Elite", "Elite", "Élite"),
        5 => tr3("Mestre", "Master", "Maestro"),
        _ => tr3("Lenda", "Legend", "Leyenda"),
    }
}
