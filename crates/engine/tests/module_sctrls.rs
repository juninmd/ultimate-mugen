use std::sync::Arc;
use ume_engine::chardef::{CharDef, Controller};
use ume_engine::matchs::{Match, StageInfo};
use ume_engine::sctrl::module::{
    module_status, rejected_by_design, ModuleStatus, MODULE_CONTROLLERS,
};

fn sctrl(kind: &str) -> Controller {
    Controller {
        kind: kind.into(),
        label: kind.into(),
        trigger_all: vec![],
        triggers: vec![],
        params: vec![],
        persistent: 1,
        ignore_hitpause: false,
        uid: 1,
    }
}

fn setup() -> (Match, usize) {
    let cd = Arc::new(CharDef::empty("p"));
    let mut m = Match::new(cd.clone(), None, StageInfo::default());
    let i = m.add_player(cd.clone(), 0, 0, 1);
    m.add_player(cd, 1, 0, 2);
    (m, i)
}

#[test]
fn every_listed_controller_is_handled_and_unknown_ones_are_not() {
    let (mut m, i) = setup();
    for (name, _) in MODULE_CONTROLLERS {
        assert!(
            ume_engine::sctrl::run(&mut m, i, &sctrl(name)),
            "{name} must be parsed and handled"
        );
    }
    assert!(!ume_engine::sctrl::run(&mut m, i, &sctrl("notacontroller")));
}

#[test]
fn unsupported_modules_warn_once() {
    let (mut m, i) = setup();
    for _ in 0..5 {
        ume_engine::sctrl::run(&mut m, i, &sctrl("storyboard"));
    }
    ume_engine::sctrl::run(&mut m, i, &sctrl("loadfile"));
    let lines: Vec<&String> = m.console.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("storyboard") && lines[0].starts_with("warning"));
    assert!(lines[1].contains("rejected by design"));
}

#[test]
fn status_table_matches_the_documented_decision() {
    assert_eq!(module_status("matchrestart"), Some(ModuleStatus::Supported));
    assert_eq!(
        module_status("savefile"),
        Some(ModuleStatus::RejectedByDesign)
    );
    assert_eq!(module_status("dialogue"), Some(ModuleStatus::Planned));
    assert_eq!(module_status("varset"), None);
    let rejected: Vec<&str> = rejected_by_design().collect();
    assert!(
        rejected.contains(&"loadfile")
            && rejected.contains(&"savefile")
            && rejected.contains(&"shaderset")
    );
}

#[test]
fn match_restart_resets_wins_and_round_at_the_next_tick() {
    let (mut m, i) = setup();
    m.wins = [1, 1];
    m.round_no = 3;
    ume_engine::sctrl::run(&mut m, i, &sctrl("matchrestart"));
    assert!(m.restart_pending);
    m.tick(&[0, 0]);
    assert!(!m.restart_pending);
    assert_eq!(m.wins, [0, 0]);
    assert_eq!(m.round_no, 1);
}
