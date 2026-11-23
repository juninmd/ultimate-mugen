use super::player::{Playback, Sprite};
use crate::app::App;
use crate::gfx::Texture2D;
use crate::registry::Src;
use std::collections::HashSet;
use std::sync::Arc;
use ume_formats::air::Animation;
use ume_formats::ini::parse_ini;
use ume_formats::screenpack::{parse_storyboard, Storyboard};
use ume_formats::vfs::{resolve, resolve_key, text, Vfs};

pub(super) fn load(app: &App, path: &str) -> Result<Playback, String> {
    let (vfs, key) =
        locate(app, path).ok_or_else(|| "file not found in imported MUGEN packs".to_string())?;
    from_vfs(&vfs, &key)
}

fn from_vfs(vfs: &Vfs, key: &str) -> Result<Playback, String> {
    let raw =
        text(resolve(vfs, "", key).ok_or_else(|| "storyboard disappeared from VFS".to_string())?);
    let storyboard = parse_storyboard(&raw);
    if storyboard.scenes.is_empty() {
        return Err("storyboard has no scenes".into());
    }
    let scene_meta = scene_meta(&raw);
    let fade_colors = scene_meta
        .iter()
        .map(|meta| (meta.fade_in, meta.fade_out))
        .collect();
    let mut storyboard = storyboard;
    for (scene, meta) in storyboard.scenes.iter_mut().zip(scene_meta) {
        scene.layer_pos = meta.layer_pos;
    }
    let mut sprites = HashSet::new();
    collect_references(&storyboard, &mut sprites);
    let decoded = match storyboard.spr.as_deref() {
        Some(name) if !sprites.is_empty() => {
            let base = key.rsplit_once('/').map_or("", |(dir, _)| dir);
            let sff_key = resolve_key(vfs, base, name)
                .ok_or_else(|| format!("sprite file '{name}' not found"))?;
            let data = vfs
                .get(&sff_key)
                .ok_or_else(|| format!("sprite file '{sff_key}' unreadable"))?;
            Some(ume_formats::sff::decode_sff_only(
                data.as_slice(),
                |g, i| sprites.contains(&(g, i)),
            )?)
        }
        _ => None,
    };
    let mut textures = std::collections::HashMap::new();
    if let Some(decoded) = decoded {
        for sprite in decoded.sprites {
            if sprite.width == 0
                || sprite.height == 0
                || sprite.width > u16::MAX as u32
                || sprite.height > u16::MAX as u32
            {
                continue;
            }
            let rgba = ume_formats::sff::sprite_to_rgba(&sprite, None);
            let texture = Texture2D::from_rgba_shared(
                sprite.width as u16,
                sprite.height as u16,
                Arc::new(rgba),
            );
            textures.insert(
                (sprite.group, sprite.index),
                Sprite {
                    texture,
                    axis: [sprite.axis_x as f32, sprite.axis_y as f32],
                },
            );
        }
    }
    Ok(Playback {
        storyboard,
        sprites: textures,
        fade_colors,
    })
}

#[derive(Default)]
struct SceneMeta {
    layer_pos: [f32; 2],
    fade_in: Option<[u8; 3]>,
    fade_out: Option<[u8; 3]>,
}

fn scene_meta(raw: &str) -> Vec<SceneMeta> {
    let mut layer_pos = [0.0; 2];
    parse_ini(raw)
        .into_iter()
        .filter(|section| {
            section.lname == "scene"
                || section
                    .lname
                    .strip_prefix("scene")
                    .is_some_and(|rest| rest.starts_with(char::is_whitespace))
        })
        .map(|section| {
            if let Some(value) = section.get("layerall.pos") {
                let mut values = value
                    .split(',')
                    .filter_map(|part| part.trim().parse::<f32>().ok());
                if let (Some(x), Some(y)) = (values.next(), values.next()) {
                    layer_pos = [x, y];
                }
            }
            SceneMeta {
                layer_pos,
                fade_in: color(section.get("fadein.col")),
                fade_out: color(section.get("fadeout.col")),
            }
        })
        .collect()
}

fn color(value: Option<&str>) -> Option<[u8; 3]> {
    let mut values = value?
        .split(',')
        .filter_map(|part| part.trim().parse::<u8>().ok());
    Some([values.next()?, values.next()?, values.next()?])
}

fn locate(app: &App, path: &str) -> Option<(Arc<Vfs>, String)> {
    let order = std::iter::once(app.sel.0).chain(0..app.reg.chars.len());
    let mut seen = HashSet::new();
    for index in order {
        let Some(entry) = app.reg.chars.get(index) else {
            continue;
        };
        let Src::Mugen { vfs, def } = &entry.src else {
            continue;
        };
        let identity = Arc::as_ptr(vfs);
        if !seen.insert(identity) {
            continue;
        }
        let base = def.rsplit_once('/').map_or("", |(dir, _)| dir);
        if let Some(key) = resolve_key(vfs, base, path) {
            return Some((vfs.clone(), key));
        }
    }
    None
}

fn collect_references(storyboard: &Storyboard, out: &mut HashSet<(i32, i32)>) {
    for scene in &storyboard.scenes {
        for layer in &scene.layers {
            let element = &layer.element;
            if let Some(sprite) = element.spr {
                out.insert(sprite);
            }
            if let Some(id) = element.anim {
                if let Some(animation) = storyboard.anims.get(&id) {
                    add_animation(out, animation);
                }
            }
        }
        if let Some(background) = scene.bg.as_deref() {
            for layer in storyboard.background(background).layers {
                if let Some(animation) = layer.anim.as_ref() {
                    add_animation(out, animation);
                } else {
                    out.insert(layer.sprite);
                }
            }
        }
    }
}

fn add_animation(out: &mut HashSet<(i32, i32)>, animation: &Animation) {
    out.extend(
        animation
            .frames
            .iter()
            .map(|frame| (frame.group, frame.index)),
    );
}

#[cfg(test)]
mod tests {
    use super::from_vfs;
    use ume_formats::vfs::{add_zip, Vfs};

    #[test]
    fn loads_real_bundled_kfm_arcade_intro_and_its_sff_layers() {
        let mut vfs = Vfs::new();
        add_zip(
            &mut vfs,
            include_bytes!("../../../../assets/mugen/elecbyte.zip"),
            "",
        )
        .unwrap();
        let playback = from_vfs(&vfs, "chars/kfm/intro.def").unwrap();
        assert_eq!(playback.storyboard.scenes.len(), 5);
        assert!(playback.storyboard.anims.contains_key(&41));
        assert!(!playback.sprites.is_empty());
        assert_eq!(playback.storyboard.scenes[0].layer_pos, [160.0, 0.0]);
        assert_eq!(playback.storyboard.scenes[1].layer_pos, [160.0, 0.0]);
        assert_eq!(playback.fade_colors[0].1, Some([255, 255, 255]));
    }
}
