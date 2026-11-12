pub(super) struct Element {
    pub stun: i32,
    pub shake: i32,
    pub palfx: Option<(i32, [i32; 3], [i32; 3])>,
}

pub(super) fn lookup(name: Option<&str>) -> Option<Element> {
    let (stun, shake, palfx) = match name? {
        "fire" => (4, 6, Some((40, [90, 20, 0], [256, 200, 160]))),
        "ice" => (16, 0, Some((70, [0, 30, 90], [200, 230, 256]))),
        "lightning" => (22, 8, Some((36, [110, 110, 0], [256, 256, 160]))),
        "water" => (8, 4, Some((50, [0, 50, 70], [220, 240, 256]))),
        "earth" => (6, 12, Some((30, [40, 25, 0], [230, 210, 170]))),
        "wind" => (4, 3, Some((14, [40, 60, 50], [240, 256, 240]))),
        _ => return None,
    };
    Some(Element { stun, shake, palfx })
}

pub(super) fn extra_stun(element: Option<&str>, stun: Option<i32>) -> i32 {
    lookup(element).map_or(0, |e| e.stun) + stun.unwrap_or(0).max(0)
}

pub(super) fn hitdef_lines(element: Option<&str>, stun: Option<i32>, explicit_shake: Option<i32>) -> String {
    let mut out = String::new();
    let found = lookup(element);
    let shake = explicit_shake.unwrap_or_else(|| found.as_ref().map_or(0, |e| e.shake).max(if stun.unwrap_or(0) > 0 { 5 } else { 0 }));
    if explicit_shake.is_none() && shake > 0 {
        out += &format!("envshake.time = {shake}\n");
    }
    if let Some((time, add, mul)) = found.and_then(|e| e.palfx) {
        out += &format!("palfx.time = {time}\npalfx.add = {}, {}, {}\npalfx.mul = {}, {}, {}\n", add[0], add[1], add[2], mul[0], mul[1], mul[2]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_element_adds_nothing() {
        assert_eq!(extra_stun(Some("void"), None), 0);
        assert!(hitdef_lines(Some("void"), None, None).is_empty());
    }

    #[test]
    fn lightning_stuns_longer_than_fire() {
        assert!(extra_stun(Some("lightning"), None) > extra_stun(Some("fire"), None));
    }

    #[test]
    fn explicit_stun_stacks_and_shakes() {
        assert_eq!(extra_stun(None, Some(10)), 10);
        assert!(hitdef_lines(None, Some(10), None).contains("envshake.time = 5"));
    }

    #[test]
    fn element_tints_the_victim() {
        assert!(hitdef_lines(Some("ice"), None, None).contains("palfx.time = 70"));
    }
}
