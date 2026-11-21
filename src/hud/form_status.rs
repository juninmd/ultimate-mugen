use ume_engine::fighter::Fighter;

pub(super) fn chip(fighter: &Fighter, ready: bool, mine: bool) -> String {
    let form = crate::i18n::tr3("FORMA", "FORM", "FORMA");
    let total = fighter.form_count() + 1;
    if ready && mine && fighter.alive {
        return format!("T · {form} {}/{total}", usize::from(fighter.form) + 2);
    }
    match fighter.next_form_cost() {
        Some(cost) => format!("{form} {}/{total} · {}", usize::from(fighter.form) + 1, cost as u32),
        None => format!("{form} {total}/{total} · MAX"),
    }
}

pub(super) fn risk(fighter: &Fighter) -> Option<String> {
    let exposed = crate::i18n::tr3("SEM GUARDA", "UNGUARDED", "SIN GUARDIA");
    let risk = (fighter.exposed_damage_multiplier() - 1.0) * 100.0;
    (risk >= 0.5).then(|| format!("+{risk:.0}% {exposed}"))
}
