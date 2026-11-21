use crate::matchs::Match;

pub(super) fn set(m: &mut Match, i: usize, not: bool, value: Option<String>, value2: Option<String>, time: i32) {
    let dst = if not { &mut m.fighters[i].not_hit_by } else { &mut m.fighters[i].hit_by };
    let put = |slot: &mut Option<(String, i32)>, attr: Option<String>| {
        if let Some(attr) = attr {
            *slot = if time > 0 && !attr.is_empty() { Some((attr, time)) } else { None };
        }
    };
    put(&mut dst[0], value);
    put(&mut dst[1], value2);
}
