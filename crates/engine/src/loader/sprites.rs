use crate::chardef::{ImageData, IndexedSprite, PalKey, Sprite};
use std::collections::HashMap;
use std::sync::Arc;
use ume_formats::act::decode_act;
use ume_formats::sff::{sprite_to_rgba, DecodedSff};

pub(super) struct SpriteTables {
    pub sprites: HashMap<(i32, i32), Sprite>,
    pub palettes: HashMap<PalKey, Arc<Vec<u8>>>,
    pub indexed: HashMap<(i32, i32), IndexedSprite>,
}

pub(super) fn sprite_tables(sff: &DecodedSff, pal_files: &[Option<Vec<u8>>], pal_no: i32, with_images: bool) -> SpriteTables {
    let pal_file = |n: i32| -> Option<&[u8]> {
        usize::try_from(n - 1).ok().and_then(|i| pal_files.get(i)).and_then(|p| p.as_deref()).filter(|a| a.len() >= 768)
    };
    let mut sprites = HashMap::new();
    let mut palettes = HashMap::new();
    let mut indexed = HashMap::new();
    sprites.reserve(sff.sprites.len());
    indexed.reserve(sff.sprites.len());
    let mut counts: HashMap<i32, usize> = HashMap::with_capacity(sff.palettes.len().max(8));
    for s in &sff.sprites {
        if s.group != 9000 {
            *counts.entry(s.palette_index).or_default() += 1;
        }
    }
    let target = counts
        .iter()
        .max_by_key(|(_, n)| **n)
        .map(|(k, _)| *k)
        .unwrap_or(-1);
    let mut over: Option<Vec<u8>> = None;
    if sff.version == 1 {
        if let Some(a) = pal_file(pal_no) {
            if a.len() >= 768 {
                over = Some(decode_act(&a));
            }
        }
    } else if pal_no != 1 {
        if let Some(p) = sff
            .palettes
            .iter()
            .find(|p| p.group == 1 && p.index == pal_no)
        {
            over = Some(p.data.to_vec());
        } else if let Some(a) =
            pal_file(pal_no)
        {
            if a.len() >= 768 {
                over = Some(decode_act(&a));
            }
        }
    }
    // v1 sprites without an embedded PCX palette rely on the character's own palette (the .act / first SFF palette)
    let fallback: Option<Vec<u8>> = over
        .clone()
        .or_else(|| {
            pal_file(1)
                .map(|a| decode_act(&a))
        })
        .or_else(|| sff.palettes.first().map(|p| p.data.to_vec()));
    let overrides_palette = |s: &ume_formats::sff::DecodedSprite|
        over.is_some() && s.group != 9000 && s.palette_index == target;
    let shared_key = |s: &ume_formats::sff::DecodedSprite| -> (i32, i32) {
        if sff.version == 1 {
            return if overrides_palette(s) || s.palette_index == target || s.palette.is_none() {
                (1, 1)
            } else {
                (-1, s.palette_index)
            };
        }
        sff.palettes
            .get(s.palette_index.max(0) as usize)
            .map(|p| (p.group, p.index))
            .unwrap_or((1, 1))
    };
    if with_images {
        for (k, p) in sff.palettes.iter().enumerate() {
            palettes.insert(
                if sff.version == 1 {
                    (-1, k as i32)
                } else {
                    (p.group, p.index)
                },
                p.data.clone(),
            );
        }
        for k in 1..=12 {
            if palettes.contains_key(&(1, k)) {
                continue;
            }
            if let Some(a) = pal_file(k)
            {
                palettes.insert((1, k), Arc::new(decode_act(&a)));
            }
        }
        let selected = over
            .clone()
            .or_else(|| fallback.clone())
            .or_else(|| palettes.get(&(1, 1)).map(|p| p.to_vec()));
        if let Some(sel) = selected {
            palettes.insert(
                if sff.version == 1 {
                    (1, 1)
                } else {
                    sff.palettes
                        .get(target.max(0) as usize)
                        .map(|p| (p.group, p.index))
                        .unwrap_or((1, 1))
                },
                Arc::new(sel),
            );
        }
    }
    let mut seen = std::collections::HashSet::with_capacity(sff.sprites.len());
    let unique: Vec<&ume_formats::sff::DecodedSprite> = sff
        .sprites
        .iter()
        .filter(|s| seen.insert((s.group, s.index)))
        .collect();
    let pal_of = |s: &ume_formats::sff::DecodedSprite| -> Option<&[u8]> {
        let paletteless = s.palette.is_none() && s.rgba.is_none();
        if paletteless {
            fallback.as_deref()
        } else if overrides_palette(s) {
            over.as_deref()
        } else {
            None
        }
    };
    let (over_arc, fallback_arc) = (over.clone().map(Arc::new), fallback.clone().map(Arc::new));
    let draw_palette = |s: &ume_formats::sff::DecodedSprite| -> Option<Arc<Vec<u8>>> {
        let paletteless = s.palette.is_none() && s.rgba.is_none();
        if paletteless {
            fallback_arc.clone()
        } else if overrides_palette(s) {
            over_arc.clone()
        } else {
            s.palette.clone()
        }
    };
    let pixels_id = |s: &ume_formats::sff::DecodedSprite| {
        s.indexed
            .as_ref()
            .map(|p| Arc::as_ptr(p) as usize)
            .or_else(|| s.rgba.as_ref().map(|p| Arc::as_ptr(p) as usize))
    };
    let mut owners: HashMap<(usize, usize, u32, u32), usize> = HashMap::new();
    let mut owner_of = Vec::with_capacity(unique.len());
    for (i, s) in unique.iter().enumerate() {
        let shared = pixels_id(s).map(|px| {
            (
                px,
                pal_of(s)
                    .or(s.palette.as_deref().map(|v| v.as_slice()))
                    .map_or(0, |p| p.as_ptr() as usize),
                s.width,
                s.height,
            )
        });
        owner_of.push(match shared {
            Some(k) => *owners.entry(k).or_insert(i),
            None => i,
        });
    }
    let renders: Vec<usize> = (0..unique.len()).filter(|&i| with_images && owner_of[i] == i && unique[i].width > 0 && unique[i].height > 0 && (unique[i].indexed.is_none() || unique[i].group == 9000)).collect();
    let images = crate::perf::time("sff:rgba", || ume_formats::par::map(&renders, |&i| ImageData::new(unique[i].width, unique[i].height, sprite_to_rgba(unique[i], pal_of(unique[i])))));
    let image_of: HashMap<usize, Arc<ImageData>> = renders.into_iter().zip(images).collect();
    for (i, s) in unique.iter().enumerate() {
        let key = (s.group, s.index);
        if let (true, Some(px)) = (with_images, s.indexed.as_ref()) {
            indexed.insert(
                key,
                IndexedSprite::new(
                    s.width,
                    s.height,
                    px.clone(),
                    shared_key(s),
                    draw_palette(s),
                ),
            );
        }
        let image = if with_images && s.width > 0 && s.height > 0 {
            image_of.get(&owner_of[i]).cloned()
        } else {
            None
        };
        sprites.insert(
            key,
            Sprite {
                image,
                width: s.width as f32,
                height: s.height as f32,
                axis_x: s.axis_x as f32,
                axis_y: s.axis_y as f32,
                scale: 1.0,
            },
        );
    }
    SpriteTables { sprites, palettes, indexed }
}
