use super::types::{LifeRange, NativeMove};

fn range_lines(owner: &str, range: &LifeRange) -> String {
    let mut out = String::new();
    if let Some(min) = range.min { out += &format!("triggerall = {owner}life >= {owner}lifemax * {min}\n"); }
    if let Some(max) = range.max { out += &format!("triggerall = {owner}life <= {owner}lifemax * {max}\n"); }
    out
}

pub(super) fn trigger_lines(m: &NativeMove) -> String {
    let Some(req) = &m.requires else { return String::new() };
    let own = req.life.as_ref().map(|r| range_lines("", r)).unwrap_or_default();
    let enemy = req.enemy_life.as_ref().map(|r| range_lines("enemy,", r)).unwrap_or_default();
    own + &enemy
}

#[cfg(test)]
mod tests {
    use super::*;

    fn move_with(requires: serde_json::Value) -> NativeMove {
        serde_json::from_value(serde_json::json!({
            "id": "m", "name": "M", "input": "x", "kind": "special", "anim": "jab", "requires": requires
        })).unwrap()
    }

    #[test]
    fn own_and_enemy_ranges_become_triggers() {
        let m = move_with(serde_json::json!({ "life": { "max": 0.5 }, "enemyLife": { "min": 0.1, "max": 0.5 } }));
        let lines = trigger_lines(&m);
        assert!(lines.contains("triggerall = life <= lifemax * 0.5\n"));
        assert!(lines.contains("triggerall = enemy,life >= enemy,lifemax * 0.1\n"));
        assert!(lines.contains("triggerall = enemy,life <= enemy,lifemax * 0.5\n"));
    }

    #[test]
    fn moves_without_requirements_add_nothing() {
        let m: NativeMove = serde_json::from_value(serde_json::json!({ "id": "m", "name": "M", "input": "x", "kind": "normal", "anim": "jab" })).unwrap();
        assert!(trigger_lines(&m).is_empty());
    }
}
