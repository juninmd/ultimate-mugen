use ume_engine::lint::{lint_air, lint_cmd, lint_cns, Rule};

fn rules(issues: &[ume_engine::lint::Issue]) -> Vec<Rule> { issues.iter().map(|i| i.rule).collect() }

#[test]
fn flags_duplicates_values_persistent_and_mismatch() {
    let text = "[Statedef -2]\n[State -2, a]\ntype = Explod\ntrigger1 = 1\npostype = nowhere\npersistent = 0\nx = 1\nx = 2\n[State 5, b]\ntype = VelSet\ntrigger1 = 1\ntrigger1 = 2\nstray text\n";
    let issues = lint_cns("a.cns", text);
    let r = rules(&issues);
    assert!(r.contains(&Rule::InvalidValue));
    assert!(r.contains(&Rule::NegativeStatePersistent));
    assert!(r.contains(&Rule::DuplicateParameter));
    assert!(r.contains(&Rule::StateNumberMismatch));
    assert!(r.contains(&Rule::GarbageLine));
    assert_eq!(r.iter().filter(|x| **x == Rule::DuplicateParameter).count(), 1);
}

#[test]
fn var_targets_share_one_slot() {
    let text = "[Statedef 1]\n[State 1, v]\ntype = VarSet\ntrigger1 = 1\nv = 1\nvalue = 1\nvar(2) = 3\n";
    let issues = lint_cns("v.cns", text);
    assert_eq!(rules(&issues), vec![Rule::DuplicateParameter]);
}

#[test]
fn clean_state_has_no_issues() {
    let text = "[Statedef 200]\ntype = S\n[State 200, ok]\ntype = Explod\ntrigger1 = 1\npostype = p1\ntrigger2 = 2\n";
    assert!(lint_cns("ok.cns", text).is_empty());
}

#[test]
fn cmd_duplicate_keys_but_same_name_blocks_allowed() {
    let text = "[Command]\nname = \"a\"\ncommand = x\ntime = 1\ntime = 2\n[Command]\nname = \"a\"\ncommand = y\n";
    assert_eq!(rules(&lint_cmd("c.cmd", text)), vec![Rule::DuplicateCommandKey]);
}

#[test]
fn air_duplicate_and_empty_actions() {
    let text = "[Begin Action 0]\n0,0, 0,0, 5\n[Begin Action 1]\n[Begin Action 0]\nClsn2: 1\n 1,0,0\n[Begin Action 2]\n1,0, 0,0, 3\n";
    let r = rules(&lint_air("a.air", text));
    assert_eq!(r, vec![Rule::EmptyAction, Rule::DuplicateAction]);
}
