use ume_formats::audit::lint::lint_text;
use ume_formats::audit::refs::{def_references, missing_in_char};
use ume_formats::audit::select_def::build_select;
use ume_formats::vfs::Vfs;

fn vfs_with(files: &[(&str, &str)]) -> Vfs {
    let mut vfs = Vfs::new();
    for (key, body) in files {
        vfs.insert(key.to_string(), body.as_bytes().to_vec());
    }
    vfs
}

#[test]
fn def_references_reads_only_file_keys_in_files_section() {
    let def = "[Info]\nname = \"x\"\n[Files]\ncmd = x.cmd\ncns = x.cns\nst = x.st\nst2 = x2.st\nstcommon = common.st\nsprite = x.sff\nanim = x.air\nsound = x.snd\nname = \"x\"\n[Music]\nlife = a.mp3\n";
    let refs: Vec<String> = def_references(def).into_iter().map(|(k, _)| k).collect();
    assert_eq!(
        refs,
        ["cmd", "cns", "st", "st2", "stcommon", "sprite", "anim", "sound"]
    );
}

#[test]
fn missing_references_resolve_like_the_loader() {
    let def = "[Files]\ncmd = x.cmd\ncns = x.cns\nsprite = x.sff\nanim = Missing.air\n";
    let vfs = vfs_with(&[
        ("c/chars/x/x.def", def),
        ("c/chars/x/x.cmd", ""),
        ("c/chars/x/x.cns", ""),
        ("c/chars/x/x.sff", ""),
    ]);
    let missing = missing_in_char(&vfs, "c/chars/x/x.def");
    assert_eq!(missing.len(), 1);
    assert_eq!(missing[0].key, "anim");
    assert_eq!(missing[0].value, "Missing.air");
}

#[test]
fn shared_asset_in_data_folder_is_found() {
    let def = "[Files]\nsound = shared.snd\n";
    let vfs = vfs_with(&[("c/chars/x/x.def", def), ("c/data/shared.snd", "")]);
    assert!(missing_in_char(&vfs, "c/chars/x/x.def").is_empty());
}

#[test]
fn lint_reports_legacy_constructs_with_suggestions() {
    let text = "[Statedef 200]\nvelset = 1, 0, 0\n[State 200, 1]\ntype = HitDef\npausetime = 10,12\nsnap = 1, 2, 3\nvolume = 255\n[State 200, 2]\ntype = Trans\n";
    let findings = lint_text("x.cns", text);
    let keys: Vec<(&str, &str)> = findings
        .iter()
        .map(|f| (f.section.as_str(), f.key.as_str()))
        .collect();
    assert_eq!(
        keys,
        [
            ("Statedef 200", "velset"),
            ("State 200, 1", "volume"),
            ("State 200, 1", "snap"),
            ("State 200, 2", "ignorehitpause"),
        ]
    );
    assert!(findings[1].message.contains("volumescale = 100"));
    assert!(findings[2].message.contains("snap = 1, 2"));
}

#[test]
fn lint_accepts_clean_state_file() {
    let text = "[Statedef 0]\nvelset = 1, 0\n[State 0, 1]\ntype = Trans\nignorehitpause = 1\n[State 0, 2]\ntype = HitDef\npausetime = 0,0\nsnap = 1, 2\n";
    assert!(lint_text("x.cns", text).is_empty());
}

#[test]
fn select_def_groups_characters_and_lists_stages() {
    let chars = vec![
        "Zed/zed.def".to_string(),
        "Ash/ash.def".to_string(),
        "loose.def".to_string(),
        "Ash/ash.def".to_string(),
    ];
    let out = build_select(
        &chars,
        &["stages/b.def".to_string(), "stages/a.def".to_string()],
        None,
    );
    assert_eq!(
        out,
        "[Characters]\n\"loose.def\", random, order=1\n;Ash\n\"Ash/ash.def\", random, order=1\n;Zed\n\"Zed/zed.def\", random, order=1\n[ExtraStages]\n\"stages/a.def\"\n\"stages/b.def\"\n"
    );
}

#[test]
fn select_def_fills_randomselect_slots_and_marks_leftovers() {
    let template = "[Characters]\nrandomselect\n-\nrandomselect\nrandomselect\n[ExtraStages]\n";
    let chars = vec!["a/a.def".to_string(), "b/b.def".to_string()];
    let out = build_select(&chars, &["stages/s.def".to_string()], Some(template));
    assert_eq!(
        out,
        "[Characters]\n\"a/a.def\", random, order=1\n-\n\"b/b.def\", random, order=1\n-\n[ExtraStages]\n"
    );
}

#[test]
fn select_def_appends_overflow_and_stages_when_template_lacks_them() {
    let template = "[Characters]\nrandomselect\n";
    let chars = vec!["a/a.def".to_string(), "b/b.def".to_string()];
    let out = build_select(&chars, &["stages/s.def".to_string()], Some(template));
    assert!(out.contains(";below are characters that do not fit"));
    assert!(out.contains("\"b/b.def\", random, order=1"));
    assert!(out.ends_with("[ExtraStages]\n\"stages/s.def\"\n"));
}
