use crate::chardef::{ImageData, Sprite};
use ume_formats::act::decode_act;
use ume_formats::air::parse_air;
use ume_formats::ini::{find_section, parse_ini_no_lines};
use ume_formats::sff::{decode_sff_only, sprite_to_rgba};
use ume_formats::vfs::{norm_path, resolve, text, Vfs};

/// Select-screen thumbnail: the 9000,1 / 9000,0 portrait, else the first idle frame (anim 0).
/// Decodes the SFF but converts a single sprite to RGBA, so it is far cheaper than a full character load and
/// the result can outlive the (size-capped) `CharDef` cache.
pub fn load_mugen_portrait(vfs: &Vfs, def_path: &str) -> Option<Sprite> {
    let def_path = norm_path(def_path);
    let base = def_path
        .rsplit_once('/')
        .map(|(b, _)| b.to_string())
        .unwrap_or_default();
    let secs = parse_ini_no_lines(&text(vfs.get(&def_path)?));
    let files = find_section(&secs, "files")?;
    let fget = |k: &str| files.get(k);
    let idle_key = fget("anim")
        .and_then(|a| resolve(vfs, &base, a))
        .and_then(|b| {
            parse_air(&text(b))
                .get(&0)
                .and_then(|a| a.frames.first())
                .map(|f| (f.group, f.index))
        });
    let sff = decode_sff_only(resolve(vfs, &base, fget("sprite")?)?, |g, i| {
        g == 9000 && (i == 0 || i == 1) || Some((g, i)) == idle_key
    })
    .ok()?;
    let find = |g: i32, i: i32| {
        sff.sprites
            .iter()
            .find(|s| s.group == g && s.index == i && s.width > 0 && s.height > 0)
    };
    let idle = || idle_key.and_then(|(g, i)| find(g, i));
    let act = fget("pal1")
        .and_then(|p| resolve(vfs, &base, &p))
        .filter(|a| a.len() >= 768)
        .map(decode_act);
    let fallback = act.or_else(|| sff.palettes.first().map(|p| p.data.to_vec()));
    // first candidate that actually has visible pixels (some packs ship blank 9000,x placeholders)
    let picked = [find(9000, 1), find(9000, 0), idle()]
        .into_iter()
        .flatten()
        .find_map(|spr| {
            let use_act = spr.group != 9000 && sff.version == 1
                || spr.palette.is_none() && spr.rgba.is_none();
            let rgba = sprite_to_rgba(spr, if use_act { fallback.as_deref() } else { None });
            if rgba.as_chunks::<4>().0.iter().all(|p| p[3] == 0) {
                return None;
            }
            Some(Sprite {
                image: Some(ImageData::new(spr.width, spr.height, rgba)),
                width: spr.width as f32,
                height: spr.height as f32,
                axis_x: spr.axis_x as f32,
                axis_y: spr.axis_y as f32,
                scale: 1.0,
            })
        });
    picked
}