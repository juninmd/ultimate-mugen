use ume_engine::chardef::{CharDef, ImageData, Sprite};
use ume_engine::native::{compile_native, NativeChar};

const SPRITES: &str = "../../assets/chars/sprites";

fn fighters_where(file: &str) -> Vec<String> {
    let mut ids: Vec<String> = std::fs::read_dir(SPRITES)
        .expect("sprites dir")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().join(file).exists())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    ids.sort();
    ids
}

fn compiled(id: &str) -> CharDef {
    compiled_pal(id, 1)
}

fn compiled_pal(id: &str, pal: i32) -> CharDef {
    std::env::remove_var("UME_FORCE_RIG");
    let json = std::fs::read_to_string(format!("../../assets/chars/{id}.json")).expect("read json");
    let nc: NativeChar = serde_json::from_str(&json).expect("parse json");
    compile_native(&nc, pal, true)
}

fn frame_art(def: &CharDef, anim: i32, frame: usize) -> &ImageData {
    let f = &def.anims[&anim].frames[frame];
    def.sprites[&(f.group, f.index)]
        .image
        .as_deref()
        .expect("sprite image")
}

fn frame_sprite(def: &CharDef, anim: i32, frame: usize) -> &Sprite {
    let f = &def.anims[&anim].frames[frame];
    &def.sprites[&(f.group, f.index)]
}

fn opaque_floor_offset(sprite: &Sprite) -> f32 {
    let image = sprite.image.as_deref().expect("sprite image");
    let bottom = image
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .enumerate()
        .filter(|(_, pixel)| pixel[3] > 0)
        .map(|(index, _)| index as u32 / image.w + 1)
        .max()
        .expect("opaque pixels");
    (bottom as f32 * sprite.height / image.h as f32 - sprite.axis_y) * sprite.scale
}

#[test]
fn every_fighter_with_a_sprite_folder_draws_it() {
    let ids = fighters_where("01_idle.png");
    assert!(
        ids.len() >= 16,
        "all sixteen native fighters have sprite folders, found {ids:?}"
    );
    for id in ids {
        let def = compiled(&id);
        assert!(
            def.sprites.values().any(|s| s.image.is_some()),
            "{id}: sprite images loaded"
        );
        let portrait = def
            .sprites
            .get(&(9000, 1))
            .and_then(|sprite| sprite.image.as_deref())
            .expect("dedicated portrait image");
        assert!(
            portrait.w.max(portrait.h) <= 256,
            "{id}: dedicated portrait fits the selection texture budget"
        );
        assert!(
            portrait
                .rgba
                .as_chunks::<4>()
                .0
                .iter()
                .any(|pixel| pixel[3] > 0),
            "{id}: dedicated portrait contains visible pixels"
        );
    }
}

#[test]
fn state_animations_use_their_own_art() {
    let ids = fighters_where("17_power2.png");
    assert!(
        ids.len() >= 15,
        "fifteen fighters ship the full pixel-art pack (kaito keeps its older one), found {ids:?}"
    );
    for id in ids {
        let def = compiled(&id);
        let idle = frame_art(&def, 0, 0);
        let states = [
            ("idle breathing", 0, 1),
            ("hit light", 5000, 0),
            ("hit medium", 5001, 0),
            ("hit heavy", 5002, 0),
            ("fall", 5050, 0),
            ("lie down", 5110, 0),
            ("transformation start", 196, 0),
            ("transformation burst", 196, 1),
            ("transformation final", 196, 2),
        ];
        for (label, anim, frame) in states {
            assert_ne!(
                frame_art(&def, anim, frame).rgba,
                idle.rgba,
                "{id}: {label} must not reuse the idle sprite"
            );
        }
        assert_ne!(
            frame_art(&def, 196, 0).rgba,
            frame_art(&def, 196, 2).rgba,
            "{id}: transformation must progress"
        );
        assert_eq!(
            frame_art(&def, 5150, 0).rgba,
            frame_art(&def, 5110, 0).rgba,
            "{id}: dead and lie down share the knocked-out art"
        );
        for (anim, frame) in [(5000, 0), (5050, 0), (5110, 0), (196, 2)] {
            let art = frame_art(&def, anim, frame);
            assert!(
                art.w > 0 && art.h > 0,
                "{id}: anim {anim} has nonzero image dimensions"
            );
            let floor_offset = opaque_floor_offset(frame_sprite(&def, anim, frame));
            assert!(floor_offset.is_finite() && floor_offset.abs() <= 1.5, "{id}: anim {anim} opaque pixels align with its own floor anchor, offset {floor_offset}");
        }
    }
}

#[test]
fn alternate_palettes_recolour_the_sprite_art() {
    for id in fighters_where("01_idle.png") {
        let base = compiled(&id);
        let swapped = compiled_pal(&id, 2);
        let before = frame_art(&base, 0, 0);
        let after = frame_art(&swapped, 0, 0);
        assert_eq!(
            (before.w, before.h),
            (after.w, after.h),
            "{id}: the palette swap keeps the sprite size"
        );
        assert!(
            before
                .rgba
                .chunks_exact(4)
                .zip(after.rgba.chunks_exact(4))
                .all(|(a, b)| a[3] == b[3]),
            "{id}: the palette swap keeps the silhouette"
        );
        let changed = before
            .rgba
            .chunks_exact(4)
            .zip(after.rgba.chunks_exact(4))
            .filter(|(a, b)| a[..3] != b[..3])
            .count();
        let opaque = before.rgba.chunks_exact(4).filter(|pixel| pixel[3] > 0).count();
        assert!(
            changed * 4 > opaque,
            "{id}: palette 2 must repaint a meaningful part of the fighter ({changed} of {opaque})"
        );
    }
}
