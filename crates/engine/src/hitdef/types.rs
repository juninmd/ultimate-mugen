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
