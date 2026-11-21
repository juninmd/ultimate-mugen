use crate::matchs::Match;

pub fn parse_sound_ref_expr(m: &Match, i: usize, raw: Option<&str>, default_own: bool) -> Option<(i32, i32, bool)> {
    let text = raw?.trim();
    if text.is_empty() {
        return None;
    }
    let (own, rest) = crate::value_ref::split(text, default_own);
    let mut parts = rest.splitn(2, ',');
    let group = m.ev(i, &crate::chardef::safe_expr(parts.next()?.trim())) as i32;
    let index = parts.next().map_or(0, |part| m.ev(i, &crate::chardef::safe_expr(part.trim())) as i32);
    Some((group, index, own))
}

pub fn parse_sound_ref(raw: Option<&str>) -> Option<(i32, i32, bool)> {
    let s = raw?.trim();
    let (own, rest) = if let Some(r) = s.strip_prefix(['s', 'S']) {
        (true, r)
    } else if let Some(r) = s.strip_prefix(['f', 'F']) {
        (false, r)
    } else {
        (false, s)
    };
    let mut it = rest.split(',').map(|x| x.trim().parse::<i32>());
    let g = it.next()?.ok()?;
    let n = it.next().and_then(|x| x.ok()).unwrap_or(0);
    Some((g, n, own))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sound_ref() {
        assert_eq!(parse_sound_ref(None), None);
        assert_eq!(parse_sound_ref(Some("")), None);
        assert_eq!(parse_sound_ref(Some("5, 0")), Some((5, 0, false)));
        assert_eq!(parse_sound_ref(Some("s5, 2")), Some((5, 2, true)));
        assert_eq!(parse_sound_ref(Some("S10, 3")), Some((10, 3, true)));
        assert_eq!(parse_sound_ref(Some("f6, 1")), Some((6, 1, false)));
        assert_eq!(parse_sound_ref(Some("F8, 4")), Some((8, 4, false)));
        assert_eq!(parse_sound_ref(Some("7")), Some((7, 0, false)));
        assert_eq!(parse_sound_ref(Some("invalid")), None);
    }
}
