//! Read-tracking probes for controller parameters.
//!
//! A parameter is "consumed" when the controller runs and asks for it through `Controller::param`. Probing it alone
//! is not enough: many controllers return early when a gating parameter (`time`, `value`, `anim`, ...) is missing,
//! which reported supported parameters such as `pause.movetime` or `helper.size.*` as unknown.
use super::*;

/// Parameters that gate a controller's early return, set to a harmless value on the second probe.
const SEED_PARAMS: [(&str, &str); 6] = [("time", "1"), ("value", "1"), ("anim", "0"), ("id", "0"), ("stateno", "0"), ("movetime", "1")];

pub fn parameter_consumed(kind: &str, name: &str) -> bool {
    if !sane_name(name) { return false; }
    cached(format!("parameter {kind} {name}"), || {
        let bare = run_controller(kind, &[(name, "0")]);
        if !bare.completed || bare.reads.contains(name) { return true; }
        let mut params: Vec<(&str, &str)> = SEED_PARAMS.iter().copied().filter(|(k, _)| *k != name).collect();
        params.push((name, "0"));
        let seeded = run_controller(kind, &params);
        !seeded.completed || seeded.reads.contains(name)
    })
}

pub fn consumed_together(kind: &str, names: &[&str]) -> Arc<HashSet<String>> {
    let mut sorted: Vec<&str> = names.iter().copied().filter(|n| sane_name(n)).collect();
    sorted.sort_unstable();
    sorted.dedup();
    let key = format!("{kind} {}", sorted.join("|"));
    if let Some(v) = CONSUMED_CACHE.with(|c| c.borrow().get(&key).cloned()) { return v; }
    let params: Vec<(&str, &str)> = sorted.iter().map(|n| (*n, "0")).collect();
    let out = run_controller(kind, &params);
    let set = Arc::new(if out.completed { out.reads } else { sorted.iter().map(|s| s.to_string()).collect() });
    CONSUMED_CACHE.with(|c| c.borrow_mut().insert(key, set.clone()));
    set
}
