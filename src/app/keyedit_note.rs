use crate::keybinds::Conflicts;

pub(crate) fn conflict_note(c: Conflicts) -> String {
    let mut note = String::new();
    if let Some(j) = c.swapped_with {
        note += &format!(
            " · {} {}",
            crate::i18n::tr3("troca com", "swapped with", "intercambiada con"),
            crate::keybinds::action_label(j)
        );
    }
    if let Some((player, j)) = c.shared_with {
        note += &format!(
            " · {} P{} ({})",
            crate::i18n::tr3(
                "ATENÇÃO: também usada por",
                "WARNING: also used by",
                "ATENCIÓN: también la usa"
            ),
            player + 1,
            crate::keybinds::action_label(j)
        );
    }
    note
}
