use std::collections::BTreeMap;
use ume_engine::chardef::CharDef;
use ume_engine::native::pixel_fit::SpriteMask;
use ume_engine::native::{compile_native, NativeChar};

const TOLERANCE: f32 = 0.75;
const MIN_BOXES: usize = 3;

fn characters() -> Vec<NativeChar> {
    let dir = std::env::var("UME_CHARS_DIR").unwrap_or_else(|_| "../../assets/chars".into());
    let mut out: Vec<NativeChar> = std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| {
            let path = e.unwrap().path();
            let stem = path.file_stem()?.to_string_lossy().to_string();
            (path.extension()? == "json" && stem != "index")
                .then(|| serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap())
        })
        .collect();
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}

fn within(inner: [f32; 4], outer: [f32; 4]) -> bool {
    inner[0] >= outer[0] - TOLERANCE
        && inner[1] >= outer[1] - TOLERANCE
        && inner[2] <= outer[2] + TOLERANCE
        && inner[3] <= outer[3] + TOLERANCE
}

fn touches_pixels(mask: &SpriteMask, b: [f32; 4]) -> bool {
    mask.trim(b)
        .is_some_and(|t| (0..4).all(|i| (t[i] - b[i]).abs() <= TOLERANCE * 4.0))
}

fn violations(id: &str, def: &CharDef) -> Vec<String> {
    let mut out = vec![];
    let mut anims: Vec<_> = def.anims.iter().collect();
    anims.sort_by_key(|(no, _)| **no);
    for (no, anim) in anims {
        for (i, frame) in anim.frames.iter().enumerate() {
            if frame.blend != ume_engine::chardef::Blend::None
                || (!frame.clsn1.is_empty() && frame.clsn1 == frame.clsn2)
            {
                continue;
            }
            let Some(sprite) = def.sprites.get(&(frame.group, frame.index)) else {
                continue;
            };
            let Some(image) = sprite.image.as_ref() else {
                continue;
            };
            let mask = SpriteMask::new(
                image,
                [sprite.axis_x, sprite.axis_y],
                sprite.scale,
                [frame.x, frame.y],
            );
            let Some(bounds) = mask.bounds() else {
                continue;
            };
            for (kind, boxes) in [("hit", &frame.clsn1), ("hurt", &frame.clsn2)] {
                if !boxes.is_empty() && boxes.len() < MIN_BOXES {
                    out.push(format!(
                        "{id}/{no}[{i}] {kind} has {} boxes, needs {MIN_BOXES}",
                        boxes.len()
                    ));
                }
            }
            for (kind, boxes) in [("hit", &frame.clsn1), ("hurt", &frame.clsn2)] {
                for b in boxes.iter() {
                    if !within(*b, bounds) {
                        out.push(format!(
                            "{id}/{no}[{i}] {kind} {b:?} leaves the visible pixels {bounds:?}"
                        ));
                    } else if !touches_pixels(&mask, *b) {
                        out.push(format!("{id}/{no}[{i}] {kind} {b:?} covers empty space"));
                    }
                }
            }
        }
    }
    out
}

#[test]
fn every_native_box_stays_on_visible_sprite_pixels() {
    let mut problems: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut checked = 0;
    for nc in characters() {
        let def = compile_native(&nc, 1, true);
        checked += def.anims.values().map(|a| a.frames.len()).sum::<usize>();
        let list = violations(&nc.id, &def);
        if !list.is_empty() {
            problems.insert(nc.id.clone(), list);
        }
    }
    for list in problems.values() {
        for line in list.iter().take(12) {
            println!("{line}");
        }
    }
    println!(
        "frames checked: {checked}, with problems: {}",
        problems.values().map(Vec::len).sum::<usize>()
    );
    assert!(problems.is_empty(), "boxes outside visible sprite pixels");
}
