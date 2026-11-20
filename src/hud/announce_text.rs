use crate::i18n::{get, Lang};
use ume_engine::matchs::{Announce, AnnounceKind};

pub(super) fn label(announce: &Announce) -> String {
    label_for(announce, get())
}

fn pick(lang: Lang, pt: &'static str, en: &'static str, es: &'static str) -> &'static str {
    match lang {
        Lang::Pt => pt,
        Lang::En => en,
        Lang::Es => es,
    }
}

fn label_for(announce: &Announce, lang: Lang) -> String {
    match announce.kind {
        AnnounceKind::Round => round(&announce.text, lang),
        AnnounceKind::Fight => pick(lang, "LUTEM!", "FIGHT!", "¡LUCHEN!").to_string(),
        AnnounceKind::Ko => ko(&announce.text, lang),
        AnnounceKind::Win => win(&announce.text, lang),
        AnnounceKind::Other => other(&announce.text, lang),
    }
}

fn round(text: &str, lang: Lang) -> String {
    if text.starts_with("FINAL") {
        return pick(lang, "ROUND FINAL", "FINAL ROUND", "RONDA FINAL").to_string();
    }
    let number = text.split_whitespace().last().unwrap_or("1");
    format!("{} {number}", pick(lang, "ROUND", "ROUND", "RONDA"))
}

fn ko(text: &str, lang: Lang) -> String {
    if text.starts_with("DOUBLE") {
        pick(lang, "K.O. DUPLO", "DOUBLE K.O.", "K.O. DOBLE").to_string()
    } else if text.starts_with("RING") {
        pick(lang, "FORA DA ARENA", "RING OUT", "FUERA DE LA ARENA").to_string()
    } else {
        "K.O.".to_string()
    }
}

fn win(text: &str, lang: Lang) -> String {
    if text == "DRAW" {
        return pick(lang, "EMPATE", "DRAW", "EMPATE").to_string();
    }
    if let Some(team) = text.strip_prefix("TEAM ").and_then(|t| t.strip_suffix(" WINS")) {
        return format!(
            "{} {team} {}",
            pick(lang, "EQUIPE", "TEAM", "EQUIPO"),
            pick(lang, "VENCE", "WINS", "GANA")
        );
    }
    match text.strip_suffix(" WINS") {
        Some(name) => format!("{name} {}", pick(lang, "VENCE", "WINS", "GANA")),
        None => text.to_string(),
    }
}

fn other(text: &str, lang: Lang) -> String {
    match text {
        "TIME OVER" => pick(lang, "TEMPO ESGOTADO", "TIME OVER", "TIEMPO AGOTADO").to_string(),
        "FINISH HIM" => pick(lang, "FINALIZE-O!", "FINISH HIM!", "¡ACÁBALO!").to_string(),
        _ => text.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn announce(text: &str, kind: AnnounceKind) -> Announce {
        Announce { text: text.into(), time: 1, total: 1, kind }
    }

    #[test]
    fn round_and_fight_are_localized() {
        let round = announce("ROUND 2", AnnounceKind::Round);
        assert_eq!(label_for(&round, Lang::Pt), "ROUND 2");
        assert_eq!(label_for(&round, Lang::En), "ROUND 2");
        assert_eq!(label_for(&round, Lang::Es), "RONDA 2");
        let last = announce("FINAL ROUND", AnnounceKind::Round);
        assert_eq!(label_for(&last, Lang::Pt), "ROUND FINAL");
        assert_eq!(label_for(&last, Lang::Es), "RONDA FINAL");
        let fight = announce("FIGHT!", AnnounceKind::Fight);
        assert_eq!(label_for(&fight, Lang::Pt), "LUTEM!");
        assert_eq!(label_for(&fight, Lang::Es), "¡LUCHEN!");
    }

    #[test]
    fn knockouts_and_wins_are_localized() {
        assert_eq!(label_for(&announce("K.O.", AnnounceKind::Ko), Lang::Pt), "K.O.");
        let double = announce("DOUBLE K.O.", AnnounceKind::Ko);
        assert_eq!(label_for(&double, Lang::Pt), "K.O. DUPLO");
        assert_eq!(label_for(&double, Lang::Es), "K.O. DOBLE");
        assert_eq!(label_for(&announce("RING OUT", AnnounceKind::Ko), Lang::Pt), "FORA DA ARENA");
        let win = announce("KAITO WINS", AnnounceKind::Win);
        assert_eq!(label_for(&win, Lang::Pt), "KAITO VENCE");
        assert_eq!(label_for(&win, Lang::Es), "KAITO GANA");
        let team = announce("TEAM NINJA WINS", AnnounceKind::Win);
        assert_eq!(label_for(&team, Lang::Pt), "EQUIPE NINJA VENCE");
        assert_eq!(label_for(&team, Lang::Es), "EQUIPO NINJA GANA");
        assert_eq!(label_for(&announce("DRAW", AnnounceKind::Win), Lang::Es), "EMPATE");
    }

    #[test]
    fn time_over_and_finish_him_are_localized() {
        let time = announce("TIME OVER", AnnounceKind::Other);
        assert_eq!(label_for(&time, Lang::Pt), "TEMPO ESGOTADO");
        assert_eq!(label_for(&time, Lang::Es), "TIEMPO AGOTADO");
        let finish = announce("FINISH HIM", AnnounceKind::Other);
        assert_eq!(label_for(&finish, Lang::Pt), "FINALIZE-O!");
        assert_eq!(label_for(&finish, Lang::Es), "¡ACÁBALO!");
    }
}
