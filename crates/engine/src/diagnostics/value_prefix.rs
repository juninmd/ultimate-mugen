use super::trigger_problem;

pub(super) fn is_value_prefix_artifact(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    let Some(rest) = lower.strip_prefix(['s', 'f']) else { return false };
    rest.is_empty() || rest.bytes().all(|b| b.is_ascii_digit()) || matches!(rest, "ifelse" | "cond") || trigger_problem(rest, None).is_none() && !rest.contains('(')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sound_source_prefixes_glued_to_the_expression_are_not_triggers() {
        for name in ["f", "s", "s8200", "F8000", "sifelse", "fvar"] {
            assert!(is_value_prefix_artifact(name), "{name}");
        }
        assert!(!is_value_prefix_artifact("somethingunknown"));
        assert!(!is_value_prefix_artifact("p2name"));
    }
}
