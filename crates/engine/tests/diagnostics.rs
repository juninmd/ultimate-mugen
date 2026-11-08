use ume_engine::diagnostics::{audit_cns, Category};

const SYNTHETIC: &str = "
[Statedef 200]
type = S
[State 200, known]
type = VelSet
trigger1 = Time = 0
x = 2
[State 200, unknown controller]
type = TotallyMadeUpController
trigger1 = 1
[State 200, unknown trigger]
type = VelSet
trigger1 = MadeUpTrigger > 3
x = 1
[State 200, unknown parameter]
type = VelSet
trigger1 = 1
x = 1
bogusparam = 5
[State 200, unknown flag]
type = AssertSpecial
trigger1 = 1
flag = notaflag
flag2 = invisible
";

#[test]
fn reports_one_unknown_of_each_kind() {
    let d = audit_cns("synthetic.cns", SYNTHETIC);
    assert_eq!(
        d.names(Category::Controller),
        vec!["totallymadeupcontroller"]
    );
    assert_eq!(d.names(Category::Trigger), vec!["madeuptrigger"]);
    assert_eq!(d.names(Category::Parameter), vec!["velset.bogusparam"]);
    assert_eq!(d.names(Category::AssertFlag), vec!["notaflag"]);
    assert!(d.names(Category::Redirection).is_empty());
    assert!(d
        .findings
        .iter()
        .all(|f| f.file == "synthetic.cns" && f.state == 200));
}

#[test]
fn summary_counts_distinct_names() {
    let d = audit_cns("synthetic.cns", SYNTHETIC);
    assert_eq!(d.summary(), "1 unsupported controllers, 1 unknown triggers, 1 unknown parameters, 1 unknown AssertSpecial flags");
    let detail = d.detail();
    assert!(detail.contains("totallymadeupcontroller"));
    assert!(detail.contains("synthetic.cns state 200"));
}

#[test]
fn repeated_unknowns_are_deduplicated() {
    let text = "[Statedef 1]\ntype = S\n[State 1, a]\ntype = Nope\ntrigger1 = 1\n[State 1, b]\ntype = Nope\ntrigger1 = 1\n";
    let d = audit_cns("dup.cns", text);
    assert_eq!(d.findings.len(), 1);
    assert_eq!(d.count(Category::Controller), 1);
}

#[test]
fn supported_features_produce_no_findings() {
    let text = "
[Statedef 0]
type = S
[State 0, a]
type = ChangeState
trigger1 = Time = 5 && Vel X > 0 && StateType = S && Command = \"x\"
trigger1 = Parent, Life > 0 || ifelse(Alive, 1, 0)
value = 200
ctrl = 1
persistent = 0
[State 0, v]
type = VarSet
trigger1 = 1
var(3) = 4
";
    let d = audit_cns("clean.cns", text);
    assert!(d.is_clean(), "{}", d.detail());
}

#[test]
fn fixture_character_has_no_unsupported_features() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/chars");
    let mut vfs = ume_formats::vfs::Vfs::new();
    ume_formats::vfs::add_dir(&mut vfs, &dir, "c").unwrap();
    let defs = ume_formats::vfs::find_char_defs(&vfs);
    assert!(!defs.is_empty());
    for def in defs {
        let character = ume_engine::loader::load_mugen_char(&vfs, &def, 1, false).unwrap();
        let d = character.diagnostics();
        assert!(d.is_clean(), "{def}: {}", d.detail());
    }
}

#[test]
fn probing_standard_names_never_panics() {
    for kind in [
        "hitdef",
        "helper",
        "explod",
        "projectile",
        "targetbind",
        "playsnd",
        "superpause",
        "destroyself",
        "changestate",
        "varset",
        "assertspecial",
        "palfx",
        "afterimage",
        "bindtoparent",
    ] {
        assert!(
            ume_engine::diagnostics::controller_supported(kind),
            "{kind}"
        );
    }
    for name in [
        "time",
        "anim",
        "movehit",
        "gethitvar",
        "numhelper",
        "p2dist",
        "random",
        "ishelper",
    ] {
        assert!(ume_engine::diagnostics::trigger_supported(name), "{name}");
    }
    assert!(!ume_engine::diagnostics::controller_supported(
        "definitelynotacontroller"
    ));
    assert!(!ume_engine::diagnostics::trigger_supported(
        "definitelynotatrigger"
    ));
}
