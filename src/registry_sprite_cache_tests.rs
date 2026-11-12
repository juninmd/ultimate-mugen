use super::{Registry, Src};
use std::sync::Arc;

fn sprite_pack(character: &str, color: tiny_skia::Color) -> Vec<u8> {
    let mut image = tiny_skia::Pixmap::new(1, 1).unwrap();
    image.fill(color);
    let png = image.encode_png().unwrap();
    let mut bytes = b"UMEP".to_vec();
    bytes.push(2);
    bytes.extend_from_slice(&(character.len() as u16).to_le_bytes());
    bytes.extend_from_slice(character.as_bytes());
    bytes.extend_from_slice(&2u32.to_le_bytes());
    for name in ["01_idle.png", "portrait.png"] {
        bytes.extend_from_slice(&(name.len() as u16).to_le_bytes());
        bytes.extend_from_slice(name.as_bytes());
        bytes.extend_from_slice(&(png.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&png);
    }
    bytes
}

#[test]
fn late_sprite_packs_replace_cached_native_definitions() {
    let mut registry = Registry::new();
    let entry = registry
        .chars
        .iter_mut()
        .find(|entry| !entry.mugen)
        .unwrap();
    let character = "late_sprite_cache_regression";
    entry.id = format!("umf:{character}");
    if let Src::Native(native) = &mut entry.src {
        Arc::make_mut(native).id = character.into();
    }
    let preview = entry.load_preview(1).unwrap();
    entry.load(2).unwrap();
    assert!(entry.peek(1).is_some());
    assert!(entry.peek(2).is_some());

    ume_engine::native::insert_sprite_pack(
        character,
        &sprite_pack(character, tiny_skia::Color::WHITE),
    )
    .unwrap();
    assert!(
        entry.peek(1).is_none(),
        "a downloaded pack must invalidate the procedural preview"
    );
    assert!(
        entry.peek(2).is_none(),
        "all cached palettes must be invalidated"
    );
    let refreshed = entry.load_preview(1).unwrap();
    assert!(!Arc::ptr_eq(&preview, &refreshed));
    assert_eq!(
        refreshed.sprites[&(9000, 1)].image.as_ref().unwrap().rgba,
        [255, 255, 255, 255].into()
    );

    let other = "unrelated_sprite_cache_regression";
    ume_engine::native::insert_sprite_pack(other, &sprite_pack(other, tiny_skia::Color::BLACK))
        .unwrap();
    assert!(Arc::ptr_eq(&refreshed, &entry.peek(1).unwrap()));

    ume_engine::native::insert_sprite_pack(
        character,
        &sprite_pack(character, tiny_skia::Color::BLACK),
    )
    .unwrap();
    let replaced = entry.load(1).unwrap();
    assert_eq!(
        replaced.sprites[&(9000, 1)].image.as_ref().unwrap().rgba,
        [0, 0, 0, 255].into()
    );
}

#[test]
fn entry_stats_does_not_load_preview_or_decode_sprites() {
    let mut reg = Registry::new();
    reg.add_bundled_mugen();
    let native_idx = reg.chars.iter().position(|c| !c.mugen).unwrap();
    let mugen_idx = reg.chars.iter().position(|c| c.mugen).unwrap();
    let (nat_atk, nat_life, nat_spd) = reg.chars[native_idx].stats();
    assert!(nat_atk > 0.0 && nat_life > 0.0 && nat_spd > 0.0);
    assert!(reg.chars[native_idx].peek(1).is_none(), "native preview must not be forced");

    let (mugen_atk, mugen_life, mugen_spd) = reg.chars[mugen_idx].stats();
    assert!(mugen_atk > 0.0 && mugen_life > 0.0 && mugen_spd > 0.0);
    assert!(reg.chars[mugen_idx].peek(1).is_none(), "mugen preview/definition must not be loaded");
}
