use ume_engine::fighter::Fighter;

pub(super) fn label(fighter: &Fighter) -> String {
    let form = crate::i18n::tr3("FORMA", "FORM", "FORMA");
    let current = fighter.form;
    let total = fighter.form_count();
    let next = fighter.next_form_cost().map_or_else(
        || "MAX".to_owned(),
        |cost| format!("T / {}", cost as u32),
    );
    if current == 0 {
        return format!("{form} 0/{total} · {next}");
    }
    let exposed = crate::i18n::tr3("SEM GUARDA", "UNGUARDED", "SIN GUARDIA");
    let risk = (fighter.exposed_damage_multiplier() - 1.0) * 100.0;
    format!("{form} {current}/{total} · {next} · +{risk:.0}% {exposed}")
}
