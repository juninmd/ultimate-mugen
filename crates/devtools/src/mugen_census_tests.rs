use super::*;

const SAMPLE: &str = "[Statedef 200]\ntype = S\n[State 200, hit]\ntype = HitDef\ndamage = 20\n[State 200, end]\ntype = ChangeState ; back\nvalue = 0\n[Statedef 5000]\ntype = S\n[State 5000, a]\ntype = ChangeState\n[Command]\nname = \"AI01\"\ncommand = a\n[Command]\nname = \"fwd\"\n[Begin Action 0]\n0,0, 0,0, 5\n";

#[test]
fn counts_states_controllers_commands_and_actions() {
    let mut census = Census::default();
    scan_text(&mut census, SAMPLE);
    assert_eq!((census.statedefs, census.common_statedefs), (2, 1));
    assert_eq!(census.controllers.get("changestate"), Some(&2));
    assert_eq!(census.controllers.get("hitdef"), Some(&1));
    assert_eq!(
        (census.commands, census.ai_commands, census.actions),
        (2, 1, 1)
    );
}

#[test]
fn report_ranks_controllers_and_maps_them_to_umf() {
    let mut census = Census {
        name: "sample".into(),
        ..Census::default()
    };
    scan_text(&mut census, SAMPLE);
    let lines = report(&[census]);
    assert!(lines[2].starts_with("| sample | 0 | "));
    assert!(lines
        .iter()
        .any(|line| line.contains("`changestate` | 2 | 67%") && line.contains("cancels")));
}
