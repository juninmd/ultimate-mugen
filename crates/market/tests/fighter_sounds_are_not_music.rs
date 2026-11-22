use ume_market::catalog::{detect, parse_tree, Kind};

#[test]
fn sound_effects_inside_a_fighter_pack_are_never_listed_as_music() {
    let tree = parse_tree(r#"{"truncated":false,"tree":[
        {"path":"umf/Goku/goku.json","type":"blob","size":9000},
        {"path":"umf/Goku/sprites/meta.json","type":"blob","size":900},
        {"path":"umf/Goku/sprites/snd_44_0.wav","type":"blob","size":79104},
        {"path":"umf/Goku/sprites/snd_33_0.wav","type":"blob","size":38564},
        {"path":"loose/snd_7_1.wav","type":"blob","size":50000},
        {"path":"music/theme.ogg","type":"blob","size":100000},
        {"path":"music/loop.wav","type":"blob","size":900000}
    ]}"#).unwrap();
    let names: Vec<String> = detect(&tree, "", "repo").items.iter().filter(|i| i.kind == Kind::Music).map(|i| i.name.clone()).collect();
    assert_eq!(names, vec!["loop", "theme"]);
}
