use super::{Reader, Scope};
use crate::chardef::Controller;

pub(super) fn try_emit(r: &mut Reader, lname: &str, sc: &Scope, out: &mut Vec<Controller>) -> bool {
    let prefix = match lname {
        "map" => "",
        "teammap" => "Team",
        "rootmap" => "Root",
        "parentmap" => "Parent",
        _ => return false,
    };
    let save = r.p;
    r.skip();
    if r.peek() != Some(b'(') {
        r.p = save;
        return false;
    }
    r.p += 1;
    let key = r.until(b")");
    if r.peek() != Some(b')') {
        r.p = save;
        return false;
    }
    r.p += 1;
    r.skip();
    let op = match (r.s.get(r.p).copied(), r.s.get(r.p + 1).copied()) {
        (Some(b':'), Some(b'=')) => {
            r.p += 2;
            "MapSet"
        }
        (Some(b'+'), Some(b'=')) => {
            r.p += 2;
            "MapAdd"
        }
        (Some(b'-'), Some(b'=')) => {
            r.p += 2;
            "MapSub"
        }
        _ => {
            r.p = save;
            return false;
        }
    };
    let expr = r.until(b";}");
    if r.peek() == Some(b';') {
        r.p += 1;
    }
    let key = key.trim().trim_matches('"').trim();
    let (ctrl, val) = if op == "MapSub" {
        (format!("{prefix}MapAdd"), format!("-({})", expr.trim()))
    } else {
        (format!("{prefix}{op}"), expr.trim().to_string())
    };
    r.emit(&ctrl, &format!("map: \"{key}\"; value: {val}"), sc, out);
    true
}
