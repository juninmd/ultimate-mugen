use ume_formats::vfs::{char_key, content_hash, stage_key, Vfs};

fn char_vfs(dir: &str, def_text: &str, sff: &[u8]) -> Vfs {
    let mut v = Vfs::new();
    v.insert(format!("{dir}/a.def"), def_text.as_bytes().to_vec());
    v.insert(format!("{dir}/a.sff"), sff.to_vec());
    v
}

const DEF: &str = "[Info]\nname = \"A\"\n[Files]\ncns = a.cns\nsprite = a.sff\n";

#[test]
fn same_character_has_same_key_regardless_of_path_case_and_comments() {
    let a = char_vfs("chars/a", DEF, b"sprites");
    let b = char_vfs("backup/copy/a", "[INFO]\nName = \"A\" ; note\n\n[FILES]\nCNS = a.cns\nSPRITE = A.SFF\n", b"sprites");
    assert_eq!(char_key(&a, "chars/a/a.def"), char_key(&b, "backup/copy/a/a.def"));
}

#[test]
fn different_sprites_or_defs_have_different_keys() {
    let a = char_vfs("chars/a", DEF, b"sprites");
    let other_sff = char_vfs("chars/a", DEF, b"other");
    let other_def = char_vfs("chars/a", &DEF.replace("\"A\"", "\"B\""), b"sprites");
    let k = char_key(&a, "chars/a/a.def");
    assert_ne!(k, char_key(&other_sff, "chars/a/a.def"));
    assert_ne!(k, char_key(&other_def, "chars/a/a.def"));
}

#[test]
fn stage_key_uses_bgdef_sprite() {
    let mk = |dir: &str| {
        let mut v = Vfs::new();
        v.insert(format!("{dir}/s.def"), b"[Info]\nname=S\n[BGdef]\nspr = s.sff\n[StageInfo]\n".to_vec());
        v.insert(format!("{dir}/s.sff"), b"art".to_vec());
        v
    };
    assert_eq!(stage_key(&mk("stages"), "stages/s.def"), stage_key(&mk("x/y"), "x/y/s.def"));
}

#[test]
fn content_hash_is_stable_and_discriminating() {
    assert_eq!(content_hash(b"zip"), content_hash(b"zip"));
    assert_ne!(content_hash(b"zip"), content_hash(b"zip2"));
}
