use crate::chardef::{portrait_pixels_usable, ImageData, Sprite};
use ume_formats::act::decode_act;
use ume_formats::air::parse_air;
use ume_formats::ini::{find_section, parse_ini_no_lines};
use ume_formats::sff::{decode_sff_only, sprite_to_rgba, DecodedSff};
use ume_formats::vfs::{norm_path, resolve, resolve_prefix, text, Vfs};

const PREFIX_BYTES: usize = 256 << 10;

pub fn load_mugen_portrait(vfs: &Vfs, def_path: &str) -> Option<Sprite> {
    let def_path = norm_path(def_path);
    let base = def_path
        .rsplit_once('/')
        .map(|(b, _)| b.to_string())
        .unwrap_or_default();
    let secs = parse_ini_no_lines(&text(vfs.get(&def_path)?));
    let files = find_section(&secs, "files")?;
    let fget = |k: &str| files.get(k);
    let spr_name = fget("sprite")?;
    let act = fget("pal1")
        .and_then(|p| resolve(vfs, &base, p))
        .filter(|a| a.len() >= 768)
        .map(|a| decode_act(&a));
    if let Some(head) = resolve_prefix(vfs, &base, spr_name, PREFIX_BYTES) {
        if let Ok(sff) = decode_sff_only(&head, |g, i| g == 9000 && (i == 0 || i == 1)) {
            if let Some(p) = pick_from(&sff, act.as_deref(), &[(9000, 1), (9000, 0)]) {
                return Some(p);
            }
        }
    }
    let sff_bytes = resolve(vfs, &base, spr_name)?;
    if let Ok(sff) = decode_sff_only(&sff_bytes, |g, i| g == 9000 && (i == 0 || i == 1)) {
        if let Some(p) = pick_from(&sff, act.as_deref(), &[(9000, 1), (9000, 0)]) {
            return Some(p);
        }
    }
    let idle_key = fget("anim")
        .and_then(|a| resolve(vfs, &base, a))
        .and_then(|b| {
            parse_air(&text(b))
                .get(&0)
                .and_then(|a| a.frames.first())
                .map(|f| (f.group, f.index))
        })?;
    let sff = decode_sff_only(&sff_bytes, |g, i| (g, i) == idle_key).ok()?;
    pick_from(&sff, act.as_deref(), &[idle_key])
}

fn pick_from(sff: &DecodedSff, act: Option<&[u8]>, keys: &[(i32, i32)]) -> Option<Sprite> {
    let fallback = act.or_else(|| sff.palettes.first().map(|p| p.data.as_slice()));
    keys.iter().copied().find_map(|(g, i)| {
        let spr = sff
            .sprites
            .iter()
            .find(|s| s.group == g && s.index == i && s.width > 0 && s.height > 0)?;
        let use_act =
            spr.group != 9000 && sff.version == 1 || spr.palette.is_none() && spr.rgba.is_none();
        let rgba = sprite_to_rgba(spr, if use_act { fallback } else { None });
        if !portrait_pixels_usable(&rgba) {
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
    })
}
