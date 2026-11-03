use super::*;

const SRC: &str = r#"
# light punch
[StateDef 200; type: S; movetype: A; physics: S; anim: 200; ctrl: 0; velset: 0, 0;]

if AnimElem = 2 {
	HitDef{ attr: S, NA; damage: 20, 0; animtype: Light; hitflag: MAF }
}
if Time = 0 {
	PlaySnd{ value: 0, 0 }
} else if Time = 5 {
	VarSet{ v: 1; value: 2 }
} else {
	VarAdd{ v: 1; value: 1 }
}
persistent(0) ignorehitpause {
	PowerAdd{ value: 10 }
}
switch Var(1) {
case 1, 2:
	CtrlSet{ value: 1 }
default:
	CtrlSet{ value: 0 }
}
let x = 3;
if AnimTime = 0 {
	ChangeState{ value: 0; ctrl: 1 }
}

[StateDef -2]
if RoundState = 2 { AssertSpecial{ flag: NoWalk } }
"#;

#[test]
fn parses_states_and_controllers() {
    let mut st = HashMap::new();
    let r = parse_zss(SRC, &mut st, true);
    assert_eq!(r.states, 2);
    assert_eq!(r.unsupported, vec!["let".to_string()]);
    let s = &st[&200];
    assert_eq!(s.param("anim").unwrap().raw, "200");
    assert_eq!(s.param("velset").unwrap().exprs.len(), 2);
    let kinds: Vec<&str> = s.controllers.iter().map(|c| c.kind.as_str()).collect();
    assert_eq!(kinds, ["hitdef", "playsnd", "varset", "varadd", "poweradd", "ctrlset", "ctrlset", "changestate"]);
    assert_eq!(s.controllers[0].raw("attr"), Some("S, NA"));
    assert_eq!(s.controllers[0].raw("damage"), Some("20, 0"));
    assert_eq!(s.controllers[2].trigger_all.len(), 2);
    assert_eq!(s.controllers[3].trigger_all.len(), 2);
    assert_eq!(s.controllers[4].persistent, 0);
    assert!(s.controllers[4].ignore_hitpause);
    assert_eq!(s.controllers[6].trigger_all.len(), 1);
    assert!(s.controllers.iter().all(|c| c.triggers.len() == 1));
    assert_eq!(st[&-2].controllers[0].kind, "assertspecial");
}

#[test]
fn appends_to_negative_states() {
    let mut st = HashMap::new();
    parse_zss("[StateDef -1]\nif 1 { CtrlSet{ value: 1 } }", &mut st, true);
    parse_zss("[StateDef -1]\nif 1 { CtrlSet{ value: 0 } }", &mut st, true);
    assert_eq!(st[&-1].controllers.len(), 2);
}

#[test]
fn skips_existing_without_override() {
    let mut st = HashMap::new();
    parse_zss("[StateDef 5]\nCtrlSet{ value: 1 }", &mut st, true);
    parse_zss("[StateDef 5]\nCtrlSet{ value: 0 }\nCtrlSet{ value: 0 }", &mut st, false);
    assert_eq!(st[&5].controllers.len(), 1);
}
