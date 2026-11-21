use crate::chardef::safe_expr;
use crate::matchs::Match;

pub(crate) fn split(raw: &str, default_own: bool) -> (bool, &str) {
    let text = raw.trim();
    match text.as_bytes().first() {
        Some(b'f' | b'F') => (false, text[1..].trim()),
        Some(b's' | b'S') => (true, text[1..].trim()),
        _ => (default_own, text),
    }
}

pub(crate) fn eval(m: &Match, i: usize, raw: &str, default_own: bool) -> (f64, bool) {
    let (own, rest) = split(raw, default_own);
    (m.ev(i, &safe_expr(rest)), own)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chardef::CharDef;
    use crate::common::{common_def, flat_stage};
    use std::sync::Arc;

    #[test]
    fn prefixes_split_and_keep_the_expression() {
        assert_eq!(split("F310+life%10", false), (false, "310+life%10"));
        assert_eq!(split("S1000", true), (true, "1000"));
        assert_eq!(split("1000", true), (true, "1000"));
        assert_eq!(split("s5, 2", false), (true, "5, 2"));
    }

    #[test]
    fn prefixed_expressions_are_evaluated() {
        let mut m = Match::new(common_def(), None, flat_stage());
        m.add_player(Arc::new(CharDef::empty("a")), 0, 0, 1);
        m.fighters[0].life = 1005.0;
        assert_eq!(eval(&m, 0, "F310+life%10", false), (315.0, false));
        assert_eq!(eval(&m, 0, "S1000", true), (1000.0, true));
        assert_eq!(eval(&m, 0, "1000", true), (1000.0, true));
    }
}
