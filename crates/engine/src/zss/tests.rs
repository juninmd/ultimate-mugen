use super::*;

fn render(n: &crate::expr::Node) -> String {
    use crate::expr::Node;
    match n {
        Node::Num(v, _) => format!("{v}"),
        Node::Un(op, a) => format!("{op}{}", render(a)),
        Node::Bin(op, a, b) => format!("{} {op} {}", render(a), render(b)),
        Node::Trig { name, args, comp, raw } => {
            let comp = comp.map(|c| format!(" {c}")).unwrap_or_default();
            if let Some(r) = raw { format!("{name}({r}){comp}") } else { format!("{name}({}){comp}", args.iter().map(render).collect::<Vec<_>>().join(", ")) }
        }
        Node::CmpStr { name, neg, values } => format!("{}{name} = {}", if *neg { "!" } else { "" }, values.join(", ")),
        Node::Redir { kind, arg, e } => format!("{kind}({})::{}", arg.as_ref().map(|a| render(a)).unwrap_or_default(), render(e)),
        _ => format!("{n:?}"),
    }
}

trait NodeText { fn node_str(&self) -> String; }
impl NodeText for crate::expr::Expr { fn node_str(&self) -> String { render(&self.node) } }

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
    assert!(r.unsupported.is_empty(), "{:?}", r.unsupported);
    assert!(r.errors.is_empty(), "{:?}", r.errors);
    let s = &st[&200];
    assert_eq!(s.param("anim").unwrap().raw, "200");
    assert_eq!(s.param("velset").unwrap().exprs.len(), 2);
    let kinds: Vec<&str> = s.controllers.iter().map(|c| c.kind.as_str()).collect();
    assert_eq!(kinds, ["hitdef", "playsnd", "varset", "varadd", "poweradd", "ctrlset", "ctrlset", "varset", "changestate"]);
    assert_eq!(s.controllers[0].raw("attr"), Some("S, NA"));
    assert_eq!(s.controllers[0].raw("damage"), Some("20, 0"));
    assert_eq!(s.controllers[2].trigger_all.len(), 2);
    assert_eq!(s.controllers[3].trigger_all.len(), 2);
    assert_eq!(s.controllers[4].persistent, 0);
    assert!(s.controllers[4].ignore_hitpause);
    assert_eq!(s.controllers[6].trigger_all.len(), 1);
    assert_eq!(s.controllers[7].raw("value"), Some("3"), "let compiles to a local var");
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

#[test]
fn let_locals_rewrite_params_and_conditions() {
    let mut st = HashMap::new();
    let r = parse_zss("[StateDef 10]\nlet x = 3;\nVarSet{ v: 2; value: x }\nif x = 3 { CtrlSet{ value: x } }", &mut st, true);
    assert!(r.errors.is_empty(), "{:?}", r.errors);
    let cs = &st[&10].controllers;
    assert_eq!(cs[0].kind, "varset");
    assert_eq!(cs[0].raw("v"), Some("59"));
    assert_eq!(cs[0].raw("value"), Some("3"));
    assert_eq!(cs[1].raw("value"), Some("var(59)"));
    assert_eq!(cs[2].trigger_all.len(), 1);
    assert_eq!(cs[2].raw("value"), Some("var(59)"));
}

#[test]
fn for_loops_unroll_with_bound_and_step() {
    let mut st = HashMap::new();
    let r = parse_zss("[StateDef 20]\nfor i = 0; i < 3; i = i + 1 { CtrlSet{ value: i } }", &mut st, true);
    assert!(r.errors.is_empty(), "{:?}", r.errors);
    let cs = &st[&20].controllers;
    let ctrlsets: Vec<_> = cs.iter().filter(|c| c.kind == "ctrlset").collect();
    assert_eq!(ctrlsets.len(), super::LOOP_CAP, "the loop is bounded");
    assert_eq!(ctrlsets[0].raw("value"), Some("var(59)"));
    let guards: Vec<String> = ctrlsets[0].trigger_all.iter().map(|e| e.node_str()).collect();
    assert!(guards.iter().any(|g| g.contains("var(59) < 3")), "{guards:?}");
    let steps = cs.iter().filter(|c| c.kind == "varset" && c.raw("v") == Some("59")).count();
    assert!(steps >= super::LOOP_CAP, "each copy advances the loop variable");
}

#[test]
fn while_loop_stops_on_its_condition() {
    let mut st = HashMap::new();
    let r = parse_zss("[StateDef 21]\nlet n = 0;\nwhile n < 2 { VarAdd{ v: 9; value: 1 } n = n + 1; }", &mut st, true);
    assert!(r.errors.is_empty(), "{:?}", r.errors);
    let cs = &st[&21].controllers;
    assert_eq!(cs.iter().filter(|c| c.kind == "varadd" && c.raw("v") == Some("9")).count(), super::LOOP_CAP);
    let guards: Vec<String> = cs.iter().filter(|c| c.kind == "varadd" && c.raw("v") == Some("9")).map(|c| c.trigger_all.iter().map(|e| e.node_str()).collect::<Vec<_>>().join(" & ")).collect();
    assert!(guards.iter().all(|g| g.contains("var(59) < 2")), "{guards:?}");
}

#[test]
fn break_and_continue_gate_the_body() {
    let mut st = HashMap::new();
    let src = "
[StateDef 22]
for i = 0; i < 5; i = i + 1 {
    if i = 2 { break; }
    CtrlSet{ value: i }
}
for j = 0; j < 3; j = j + 1 {
    continue;
    PowerAdd{ value: 1 }
}
";
    let r = parse_zss(src, &mut st, true);
    assert!(r.errors.is_empty(), "{:?}", r.errors);
    let cs = &st[&22].controllers;
    assert!(cs.iter().any(|c| c.kind == "varset" && c.raw("value") == Some("1")), "break sets its flag");
    let ctrlsets: Vec<_> = cs.iter().filter(|c| c.kind == "ctrlset").collect();
    assert!(ctrlsets.iter().all(|c| c.trigger_all.iter().any(|e| e.node_str().contains("var(56)") || e.node_str().contains("var(57)") || e.node_str().contains("var(58)"))));
    let powers: Vec<_> = cs.iter().filter(|c| c.kind == "poweradd").collect();
    assert!(!powers.is_empty());
    assert!(powers.iter().all(|c| c.trigger_all.iter().any(|e| e.node_str().contains("= 0"))), "code after continue is gated by the skip flag");
}

#[test]
fn functions_inline_and_check_arguments() {
    let mut st = HashMap::new();
    let src = "
[Function double(n)]
{ VarSet{ v: 1; value: n } }
[StateDef 30]
call double(4);
";
    let r = parse_zss(src, &mut st, true);
    assert!(r.errors.is_empty(), "{:?}", r.errors);
    let cs = &st[&30].controllers;
    assert_eq!(cs[0].raw("value"), Some("4"), "the argument is bound to the parameter local");
    assert_eq!(cs[1].raw("value"), Some("var(59)"));
    let r = parse_zss("[StateDef 31]\ncall double();", &mut st, true);
    assert!(!r.errors.is_empty(), "wrong arity is a compile error");
    let r = parse_zss("[StateDef 32]\ncall missing(1);", &mut st, true);
    assert!(r.errors.iter().any(|e| e.contains("unknown function")), "{:?}", r.errors);
}

#[test]
fn a_compile_error_skips_the_file() {
    let mut st = HashMap::new();
    parse_state_file("broken.zss", "[StateDef 40]\ncall missing(1);", &mut st, true);
    assert!(!st.contains_key(&40), "the state is not loaded when compilation fails");
}
