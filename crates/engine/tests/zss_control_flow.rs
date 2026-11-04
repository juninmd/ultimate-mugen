mod support;
use std::sync::Arc;
use support::*;
use ume_engine::chardef::CharDef;
use ume_engine::cns::parse_states;
use ume_engine::matchs::Match;
use ume_engine::zss::{parse_state_file, parse_zss, LOOP_ITERATION_CAP};

fn zss_char(src: &str) -> Arc<CharDef> {
    let mut d = Arc::try_unwrap(mugen_char("z", "")).ok().unwrap();
    let r = parse_zss(src, &mut d.states, true);
    assert!(r.errors.is_empty(), "{:?}", r.errors);
    Arc::new(d)
}

fn run_state(d: Arc<CharDef>, state: i32) -> Match {
    let mut m = duel(d, mugen_char("p2", ""));
    enter(&mut m, 0, state);
    m.tick(&[0, 0]);
    m
}

const FUNCTION_ZSS: &str = r#"
[Function bump(n)]
let i = 0;
let total = 0;
while i < n {
	i++;
	if i = 3 { continue }
	if i > 6 { break }
	total += i;
}
return total;

[StateDef 1000]
let r = call bump(5);
VarSet{ v: 1; value: r }

[StateDef 1001]
let r = call bump(10);
VarSet{ v: 1; value: r }

[StateDef 1002]
call bump(2);
let r = call bump(2);
VarSet{ v: 1; value: r + 100 }
"#;

const FUNCTION_CNS: &str = r#"
[Statedef 1000]
[State 1000, a]
type = VarSet
trigger1 = 1
v = 1
value = 12

[Statedef 1001]
[State 1001, a]
type = VarSet
trigger1 = 1
v = 1
value = 18

[Statedef 1002]
[State 1002, a]
type = VarSet
trigger1 = 1
v = 1
value = 103
"#;

#[test]
fn function_used_in_three_states_matches_the_cns_equivalent() {
    let zss = zss_char(FUNCTION_ZSS);
    let cns = mugen_char("c", FUNCTION_CNS);
    for state in [1000, 1001, 1002] {
        let from_zss = run_state(zss.clone(), state).fighters[0].vars[1];
        let from_cns = run_state(cns.clone(), state).fighters[0].vars[1];
        assert_eq!(from_zss, from_cns, "state {state}");
    }
}

#[test]
fn for_loop_accumulates_with_locals() {
    let d = zss_char("[StateDef 1000]\nlet acc = 0;\nfor i = 0; i < 4; i++ {\n\tacc += i;\n}\nVarSet{ v: 2; value: acc }\n");
    assert_eq!(run_state(d, 1000).fighters[0].vars[2], 6);
}

#[test]
fn dollar_prefixed_locals_and_component_names_do_not_clash() {
    let d = zss_char("[StateDef 1000]\nlet x = 7;\nlet $y = $x * 2;\nVarSet{ v: 3; value: y + x }\nif vel x = 0 { VarSet{ v: 4; value: 1 } }\n");
    let m = run_state(d, 1000);
    assert_eq!(m.fighters[0].vars[3], 21);
    assert_eq!(m.fighters[0].vars[4], 1);
}

#[test]
fn float_locals_keep_their_fraction() {
    let d = zss_char("[StateDef 1000]\nlet h = 1.5;\nh *= 3;\nVarSet{ fv: 0; value: h }\n");
    assert_eq!(run_state(d, 1000).fighters[0].fvars[0], 4.5);
}

#[test]
fn if_conditions_use_locals_and_nested_scopes_are_released() {
    let d = zss_char("[StateDef 1000]\nlet a = 2;\nif a = 2 {\n\tlet b = a + 5;\n\tVarSet{ v: 5; value: b }\n} else {\n\tVarSet{ v: 5; value: -1 }\n}\n");
    assert_eq!(run_state(d, 1000).fighters[0].vars[5], 7);
}

#[test]
fn runaway_loops_stop_at_the_iteration_cap() {
    let d = zss_char("[StateDef 1000]\nlet i = 0;\nwhile 1 {\n\ti++;\n}\nVarSet{ v: 6; value: i }\n");
    assert_eq!(run_state(d, 1000).fighters[0].vars[6], LOOP_ITERATION_CAP as i32 + 1);
}

#[test]
fn function_results_feed_later_calls() {
    let src = "[Function twice(v)]\nreturn v * 2;\n\n[StateDef 1000]\nlet a = call twice(4);\nlet b = call twice(a);\nVarSet{ v: 7; value: b }\n";
    assert_eq!(run_state(zss_char(src), 1000).fighters[0].vars[7], 16);
}

#[test]
fn early_return_skips_the_rest_of_the_function() {
    let src = "[Function pick(v)]\nif v > 3 { return 10; }\nreturn 20;\n\n[StateDef 1000]\nlet a = call pick(5);\nlet b = call pick(1);\nVarSet{ v: 8; value: a + b }\n";
    assert_eq!(run_state(zss_char(src), 1000).fighters[0].vars[8], 30);
}

fn errors_of(src: &str) -> Vec<String> { parse_zss(src, &mut Default::default(), true).errors }

#[test]
fn unsupported_constructs_are_compile_errors() {
    assert!(!errors_of("[StateDef 1]\nbreak;").is_empty());
    assert!(!errors_of("[StateDef 1]\ncontinue;").is_empty());
    assert!(!errors_of("[StateDef 1]\nreturn 1;").is_empty());
    assert!(!errors_of("[StateDef 1]\ncall nothing(1);").is_empty());
    assert!(!errors_of("[StateDef 1]\nlet a = $missing;").is_empty());
    assert!(!errors_of("[StateDef 1]\nwhile {").is_empty());
    assert!(!errors_of("[Remap 1]\nx").is_empty());
    assert!(!errors_of("[Function f(a)]\ncall f(a);\n[StateDef 1]\ncall f(1);").is_empty());
    assert!(!errors_of("[Function f(a)]\nreturn a;\n[StateDef 1]\ncall f(1, 2);").is_empty());
    assert!(!errors_of("[Function f(a)]\nreturn a;\n[StateDef 1]\nVarSet{ v: 1; value: f(2) }").is_empty());
    assert!(errors_of("[StateDef 1]\nlet a = 1;\nwhile a < 3 { a++; }").is_empty());
}

#[test]
fn state_file_loading_reports_compile_errors() {
    let mut states = Default::default();
    assert!(parse_state_file("bad.zss", "[StateDef 1]\nbreak;", &mut states, true).is_err());
    assert!(parse_state_file("ok.zss", "[StateDef 1]\nVarSet{ v: 1; value: 1 }", &mut states, true).is_ok());
    let mut cns_states = Default::default();
    parse_states("[Statedef 1]", &mut cns_states, true);
    assert!(parse_state_file("a.cns", "[Statedef 1]", &mut cns_states, true).is_ok());
}
