mod support;
use std::collections::BTreeSet;
use support::gogeta::*;

#[test]
fn sweep() {
    let Some(d) = load(1) else { return };
    let mut p2_all: BTreeSet<i32> = BTreeSet::new();
    let mut p1_all: BTreeSet<i32> = BTreeSet::new();
    let mut anomalies = 0;
    for c in d.commands.iter() {
        if c.name.starts_with("ai") {
            continue;
        }
        for (label, gap, air) in [
            ("near", 40.0, false),
            ("mid", 120.0, false),
            ("far", 300.0, false),
            ("air", 60.0, true),
            ("airfar", 200.0, true),
        ] {
            let o = run_command(&d, c, gap, air, 150);
            p2_all.extend(o.p2_states.iter().copied());
            p1_all.extend(o.p1_states.iter().copied());
            if !o.p1_recovered || !o.p2_recovered || !o.command_fired {
                anomalies += 1;
                println!("ANOMALY {:<14} {label:<6} fired={} rec1={} rec2={} fin1={:?} fin2={:?} p1={:?} p2={:?}", c.name, o.command_fired, o.p1_recovered, o.p2_recovered, o.p1_final, o.p2_final, o.p1_states, o.p2_states);
            }
        }
    }
    assert_eq!(anomalies, 0);
    println!("P1 STATES {:?}", p1_all);
    println!("P2 STATES {:?}", p2_all);
    let mut all: BTreeSet<i32> = d.states.keys().copied().collect();
    for s in &p1_all {
        all.remove(s);
    }
    println!("UNREACHED {} of {}: {:?}", all.len(), d.states.len(), all);
}
