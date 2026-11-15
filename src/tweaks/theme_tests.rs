use super::*;
use crate::tweaks::{parse, Tweaks};

#[test]
fn roles_remap_their_stock_colour_and_nothing_else() {
    let (tweaks, _, warnings) = parse("[theme]\naccent = \"#00aa55\"\ntitle = \"#ffffff\"\n");
    assert!(warnings.is_empty());
    install(&Tweaks::default());
    assert_eq!((remap(0xffcc33), remap(0xff4d6d)), (0xffcc33, 0xff4d6d));
    install(&tweaks);
    assert_eq!(remap(0xff4d6d), 0x00aa55);
    assert_eq!(remap(0xff4d6e), 0x00aa55);
    assert_eq!(remap(0xffcc33), 0xffffff);
    assert_eq!(remap(0x22e8ff), 0x22e8ff);
    assert_eq!(remap(0x123456), 0x123456);
    install(&Tweaks::default());
}

#[test]
fn a_preset_sets_many_values_and_later_keys_still_win() {
    let (tweaks, _, warnings) = parse("[theme]\ntitle = \"#010203\"\npreset = \"emerald\"\n");
    assert!(warnings.is_empty());
    assert_eq!(tweaks.title, [1, 2, 3]);
    assert_ne!(tweaks.accent, Tweaks::default().accent);
    assert_ne!(tweaks.life, Tweaks::default().life);
    let (stock, _, _) = parse("[theme]\npreset = \"default\"\n");
    assert_eq!(stock, Tweaks::default());
    let (_, _, warnings) = parse("[theme]\npreset = \"nope\"\n");
    assert!(warnings[0].contains("emerald") && warnings[0].contains("sunset"));
}

#[test]
fn music_section_maps_stages_and_falls_back_to_fight() {
    let (_, labels, warnings) = parse(
        "[music]\nfight = \"music/main.ogg\"\nDojo = \"music/dojo.mp3\"\nbad = \"notes.txt\"\n",
    );
    assert_eq!(warnings.len(), 1);
    let lookup = |key: &str| {
        labels
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, path)| path.clone())
    };
    assert_eq!(
        crate::tweaks::music_path("builtin:dojo", lookup).as_deref(),
        Some("music/dojo.mp3")
    );
    assert_eq!(
        crate::tweaks::music_path("volcano", lookup).as_deref(),
        Some("music/main.ogg")
    );
    assert_eq!(crate::tweaks::music_path("volcano", |_| None), None);
}

#[test]
fn life_bar_keeps_stock_gradient_until_a_colour_is_chosen() {
    let stock = Tweaks::default();
    let (top, bottom) = pair(stock.life, stock.life, LIFE_TOP, 1.0);
    assert_eq!((top.r, bottom.g), (1.0, stock.life[1] as f32 / 255.0));
    let (top, bottom) = pair([0, 100, 200], stock.life, LIFE_TOP, 1.0);
    assert!(top.b > bottom.b && top.r > bottom.r);
}
