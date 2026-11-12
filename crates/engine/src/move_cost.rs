use crate::chardef::{Controller, StateDef};
use crate::expr::Node;
use std::collections::{HashMap, HashSet};

pub fn cost_of(states: &HashMap<i32, StateDef>, command: &str) -> f32 {
    let mut targets = HashSet::new();
    for sd in states.iter().filter(|(id, _)| **id < 0).map(|(_, s)| s) {
        for c in sd.controllers.iter().filter(|c| c.kind == "changestate") {
            if names_command(c, command) {
                if let Some(v) = c
                    .param("value")
                    .and_then(|p| p.exprs.first())
                    .and_then(|e| literal(&e.node))
                {
                    targets.insert(v as i32);
                }
            }
        }
    }
    if targets.len() != 1 {
        return 0.0;
    }
    let target = targets.into_iter().next().unwrap_or(-1);
    let Some(sd) = states.get(&target) else {
        return 0.0;
    };
    let spent: f64 = sd
        .controllers
        .iter()
        .filter(|c| c.kind == "poweradd" && at_entry(c))
        .filter_map(|c| {
            c.param("value")
                .and_then(|p| p.exprs.first())
                .and_then(|e| literal(&e.node))
        })
        .filter(|v| *v < 0.0)
        .sum();
    (-spent) as f32
}

fn names_command(c: &Controller, command: &str) -> bool {
    c.trigger_all
        .iter()
        .chain(c.triggers.iter().flatten())
        .any(|e| contains_command(&e.node, command))
}

fn contains_command(n: &Node, command: &str) -> bool {
    match n {
        Node::CmpStr {
            name,
            neg: false,
            values,
        } => name == "command" && values.iter().any(|v| v == command),
        Node::Bin(_, a, b) => contains_command(a, command) || contains_command(b, command),
        Node::Un(_, a) => contains_command(a, command),
        _ => false,
    }
}

fn at_entry(c: &Controller) -> bool {
    c.trigger_all
        .iter()
        .chain(c.triggers.iter().flatten())
        .any(|e| is_time_zero(&e.node))
}

fn is_time_zero(n: &Node) -> bool {
    match n {
        Node::Bin("=", a, b) => {
            matches!((a.as_ref(), b.as_ref()), (Node::Trig { name, .. }, Node::Num(v, _)) | (Node::Num(v, _), Node::Trig { name, .. }) if name == "time" && *v == 0.0)
        }
        Node::Bin("&&", a, b) => is_time_zero(a) || is_time_zero(b),
        _ => false,
    }
}

fn literal(n: &Node) -> Option<f64> {
    match n {
        Node::Num(v, _) => Some(*v),
        Node::Un("-", a) => literal(a).map(|v| -v),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn states(text: &str) -> HashMap<i32, StateDef> {
        let mut m = HashMap::new();
        crate::cns::parse_states(text, &mut m, true);
        m
    }

    #[test]
    fn cost_comes_from_entry_power_add_of_the_command_target() {
        let s = states("[Statedef -1]\n[State -1, a]\ntype = ChangeState\ntrigger1 = command = \"QCF_x\"\nvalue = 200\nctrl = 1\n\n[Statedef 200]\ntype = S\nctrl = 0\n[State 200, p]\ntype = PowerAdd\ntrigger1 = Time = 0\nvalue = -1000\n");
        assert_eq!(cost_of(&s, "QCF_x"), 1000.0);
        assert_eq!(cost_of(&s, "QCF_y"), 0.0);
    }

    #[test]
    fn unknown_or_drain_costs_are_not_guessed() {
        let s = states("[Statedef -1]\n[State -1, a]\ntype = ChangeState\ntrigger1 = command = \"x\"\nvalue = 300\n\n[Statedef 300]\ntype = S\n[State 300, p]\ntype = PowerAdd\ntrigger1 = 1\nvalue = -20\n");
        assert_eq!(cost_of(&s, "x"), 0.0);
    }
}
