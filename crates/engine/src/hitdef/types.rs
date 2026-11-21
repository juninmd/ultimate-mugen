pub(super) fn animtype(s: &str) -> Option<i32> {
    let s = s.trim();
    if s.eq_ignore_ascii_case("light") {
        Some(0)
    } else if s.eq_ignore_ascii_case("medium") || s.eq_ignore_ascii_case("med") {
        Some(1)
    } else if s.eq_ignore_ascii_case("hard") || s.eq_ignore_ascii_case("heavy") {
        Some(2)
    } else if s.eq_ignore_ascii_case("back") {
        Some(3)
    } else if s.eq_ignore_ascii_case("up") {
        Some(4)
    } else if s.eq_ignore_ascii_case("diagup") {
        Some(5)
    } else {
        None
    }
}
pub(super) fn priority_type(s: Option<&str>) -> i32 {
    s.and_then(|v| v.split(',').nth(1))
        .map(|v| v.trim().to_ascii_lowercase())
        .map_or(0, |v| match v.as_str() {
            "miss" => 1,
            "dodge" => 2,
            _ => 0,
        })
}

pub(super) fn groundtype(s: &str) -> Option<i32> {
    let s = s.trim();
    if s.eq_ignore_ascii_case("none") {
        Some(0)
    } else if s.eq_ignore_ascii_case("high") {
        Some(1)
    } else if s.eq_ignore_ascii_case("low") {
        Some(2)
    } else if s.eq_ignore_ascii_case("trip") {
        Some(3)
    } else {
        None
    }
}