use super::chain::ChainPlan;
use super::types::NativeMove;

pub(super) fn lead(m: &NativeMove, plan: &ChainPlan, index: usize) -> String {
    let Some(spec) = &m.rekka else { return format!("trigger1 = ctrl{}", plan.triggers(index)) };
    let contact = if spec.contact { " && movecontact" } else { "" };
    let lines: Vec<String> = spec.after.iter().filter_map(|id| plan.sid_of(id)).enumerate().map(|(n, sid)| {
        format!("trigger{} = stateno = {sid} && time >= {} && time <= {}{contact}", n + 1, spec.window[0], spec.window[1])
    }).collect();
    if lines.is_empty() { String::from("trigger1 = 0") } else { lines.join("\n") }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn mv(value: serde_json::Value) -> NativeMove { serde_json::from_value(value).unwrap() }

    #[test]
    fn follow_up_only_fires_from_its_source_inside_the_window() {
        let first = mv(json!({"id": "first", "name": "first", "input": "~D, DF, F, x", "kind": "special", "anim": "a"}));
        let second = mv(json!({"id": "second", "name": "second", "input": "x", "kind": "special", "anim": "a", "rekka": {"after": ["first"], "window": [8, 24]}}));
        let plan = ChainPlan::new([first.clone(), second.clone()].iter());
        let gate = lead(&second, &plan, 1);
        assert!(gate.contains("stateno = 1000") && gate.contains("time >= 8") && gate.contains("time <= 24"));
        assert!(!gate.contains("ctrl"), "a follow-up cannot start from neutral");
        assert!(!gate.contains("movecontact"), "contact is optional");
        assert!(lead(&first, &plan, 0).starts_with("trigger1 = ctrl"));
    }

    #[test]
    fn contact_gate_and_unknown_sources() {
        let first = mv(json!({"id": "first", "name": "first", "input": "~D, DF, F, x", "kind": "special", "anim": "a"}));
        let gated = mv(json!({"id": "g", "name": "g", "input": "y", "kind": "special", "anim": "a", "rekka": {"after": ["first"], "window": [1, 9], "contact": true}}));
        let orphan = mv(json!({"id": "o", "name": "o", "input": "y", "kind": "special", "anim": "a", "rekka": {"after": ["missing"], "window": [1, 9]}}));
        let plan = ChainPlan::new([first, gated.clone(), orphan.clone()].iter());
        assert!(lead(&gated, &plan, 1).contains("movecontact"));
        assert_eq!(lead(&orphan, &plan, 2), "trigger1 = 0");
    }
}
