//! Build a CharDef from a MUGEN character inside a VFS.
use crate::chardef::{default_constants, CharDef, ImageData, IndexedSprite, MoveInfo, Source, Sprite};
use crate::cns::{parse_cmd, parse_constants, parse_states, scale_default_constants};
use crate::commands::parse_command;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use ume_formats::act::decode_act;
use ume_formats::air::parse_air;
use ume_formats::ini::{find_section, nums, parse_ini, unquote};
use ume_formats::sff::{decode_sff, sprite_to_rgba};
use ume_formats::snd::decode_snd;
use ume_formats::vfs::{norm_path, resolve, text, Vfs};

pub fn load_mugen_char(vfs: &Vfs, def_path: &str, pal_no: i32, with_images: bool) -> Result<CharDef, String> {
    let def_path = norm_path(def_path);
    let base = def_path.rsplit_once('/').map(|(b, _)| b.to_string()).unwrap_or_default();
    let dt = text(vfs.get(&def_path).ok_or("def not found")?);
    let secs = parse_ini(&dt);
    let info = find_section(&secs, "info");
    let files = find_section(&secs, "files");
    let fget = |k: &str| files.and_then(|s| s.get(k)).map(|s| s.to_string());
    let name = info.and_then(|s| s.get("name")).map(unquote).filter(|s| !s.is_empty()).unwrap_or(def_path.clone());
    let display = info.and_then(|s| s.get("displayname")).map(unquote).filter(|s| !s.is_empty()).unwrap_or(name.clone());
    let author = info.and_then(|s| s.get("author")).map(unquote).unwrap_or_default();
    let lc = nums(info.and_then(|s| s.get("localcoord")), &[320.0, 240.0]);
    let local_w = if lc[0] > 0.0 { lc[0] } else { 320.0 };

    let mut constants = default_constants();
    let mut provided = HashSet::new();
    let mut states = HashMap::new();
    let cns_name = fget("cns").unwrap_or_default();
    let cns_text = resolve(vfs, &base, &cns_name).map(text).unwrap_or_default();
    parse_constants(&cns_text, &mut constants, &mut provided);
    scale_default_constants(&mut constants, &provided, local_w as f64 / 320.0);
    parse_states(&cns_text, &mut states, true);
    for k in ["st", "st0", "st1", "st2", "st3", "st4", "st5", "st6", "st7", "st8", "st9"] {
        let Some(fname) = fget(k) else { continue };
        if norm_path(&unquote(&fname)) == norm_path(&unquote(&cns_name)) { continue; }
        if let Some(b) = resolve(vfs, &base, &fname) { crate::zss::parse_state_file(&fname, &text(b), &mut states, true); }
    }
    let cmd_text = resolve(vfs, &base, &fget("cmd").unwrap_or_default()).map(text).unwrap_or_default();
    let (mut commands, cmd_states) = parse_cmd(&cmd_text);
    for (id, sd) in cmd_states {
        if id < 0 && states.contains_key(&id) { states.get_mut(&id).map(|s: &mut crate::chardef::StateDef| s.controllers.extend(sd.controllers)); } else { states.insert(id, sd); }
    }
    if let Some(stc) = fget("stcommon") {
        if let Some(b) = vfs.get(&norm_path(&format!("{base}/{}", unquote(&stc)))) { crate::zss::parse_state_file(&stc, &text(b), &mut states, false); }
    }
    for (n, c) in [("holdfwd", "/$F"), ("holdback", "/$B"), ("holdup", "/$U"), ("holddown", "/$D")] {
        if !commands.iter().any(|x| x.name == n) { commands.push(parse_command(n, c, 1, 1)); }
    }
    let anims = resolve(vfs, &base, &fget("anim").unwrap_or_default()).map(|b| parse_air(&text(b))).unwrap_or_default();

    let mut sprites = HashMap::new();
    let mut palettes = HashMap::new();
    let mut indexed = HashMap::new();
    if let Some(buf) = resolve(vfs, &base, &fget("sprite").unwrap_or_default()) {
        let sff = decode_sff(buf)?;
        let mut counts: HashMap<i32, usize> = HashMap::new();
        for s in &sff.sprites { if s.group != 9000 { *counts.entry(s.palette_index).or_default() += 1; } }
        let target = counts.iter().max_by_key(|(_, n)| **n).map(|(k, _)| *k).unwrap_or(-1);
        let mut over: Option<Vec<u8>> = None;
        if sff.version == 1 {
            if let Some(a) = fget(&format!("pal{pal_no}")).and_then(|p| resolve(vfs, &base, &p)) { if a.len() >= 768 { over = Some(decode_act(a)); } }
        } else if pal_no != 1 {
            if let Some(p) = sff.palettes.iter().find(|p| p.group == 1 && p.index == pal_no) { over = Some(p.data.to_vec()); }
            else if let Some(a) = fget(&format!("pal{pal_no}")).and_then(|p| resolve(vfs, &base, &p)) { if a.len() >= 768 { over = Some(decode_act(a)); } }
        }
        // v1 sprites without an embedded PCX palette rely on the character's own palette (the .act / first SFF palette)
        let fallback: Option<Vec<u8>> = over.clone().or_else(|| fget("pal1").and_then(|p| resolve(vfs, &base, &p)).filter(|a| a.len() >= 768).map(decode_act))
            .or_else(|| sff.palettes.first().map(|p| p.data.to_vec()));
        let shared_key = |s: &ume_formats::sff::DecodedSprite| -> (i32, i32) {
            if sff.version == 1 { return if s.palette_index == target || s.palette.is_none() { (1, 1) } else { (-1, s.palette_index) }; }
            sff.palettes.get(s.palette_index.max(0) as usize).map(|p| (p.group, p.index)).unwrap_or((1, 1))
        };
        if with_images {
            for (k, p) in sff.palettes.iter().enumerate() { palettes.insert(if sff.version == 1 { (-1, k as i32) } else { (p.group, p.index) }, p.data.clone()); }
            for k in 1..=12 {
                if palettes.contains_key(&(1, k)) { continue; }
                if let Some(a) = fget(&format!("pal{k}")).and_then(|p| resolve(vfs, &base, &p)).filter(|a| a.len() >= 768) { palettes.insert((1, k), Arc::new(decode_act(a))); }
            }
            let selected = over.clone().or_else(|| fallback.clone()).or_else(|| palettes.get(&(1, 1)).map(|p| p.to_vec()));
            if let Some(sel) = selected { palettes.insert(if sff.version == 1 { (1, 1) } else { sff.palettes.get(target.max(0) as usize).map(|p| (p.group, p.index)).unwrap_or((1, 1)) }, Arc::new(sel)); }
        }
        for s in &sff.sprites {
            let key = (s.group, s.index);
            if sprites.contains_key(&key) { continue; }
            if let (true, Some(px)) = (with_images, s.indexed.as_ref()) { indexed.insert(key, IndexedSprite { w: s.width, h: s.height, pixels: Arc::new(px.clone()), pal: shared_key(s) }); }
            let paletteless = s.palette.is_none() && s.rgba.is_none();
            let pal = if paletteless { fallback.as_deref() } else if over.is_some() && s.palette_index == target && s.group != 9000 { over.as_deref() } else { None };
            let image = if with_images && s.width > 0 && s.height > 0 { Some(ImageData::new(s.width, s.height, sprite_to_rgba(s, pal))) } else { None };
            sprites.insert(key, Sprite { image, width: s.width as f32, height: s.height as f32, axis_x: s.axis_x as f32, axis_y: s.axis_y as f32, scale: 1.0 });
        }
    }
    let sounds = resolve(vfs, &base, &fget("sound").unwrap_or_default()).map(decode_snd).unwrap_or_default();
    let ai_commands = commands.iter().map(|c| c.name.clone())
        .filter(|n| { let l = n.to_ascii_lowercase(); !(l.starts_with("hold") || l.starts_with("recovery") || ["fwd", "back", "up", "down", "ff", "bb"].contains(&l.as_str())) }).collect();
    let mut d = CharDef::empty(&format!("mugen:{def_path}"));
    d.name = name; d.display_name = display; d.author = author; d.source = Source::Mugen;
    d.local_w = local_w; d.local_h = lc[1];
    d.mugen_version = crate::chardef::parse_mugen_version(info.and_then(|s| s.get("mugenversion")));
    d.constants = constants; d.portrait = sprites.get(&(9000, 0)).cloned(); d.sprites = sprites; d.anims = anims;
    d.commands = commands; d.states = states; d.sounds = sounds; d.ai_commands = ai_commands; d.series = "M.U.G.E.N".into();
    d.palettes = palettes; d.indexed = indexed;
    let _ = Vec::<MoveInfo>::new;
    let _ = Arc::new(0);
    Ok(d)
}

/// Select-screen thumbnail: the 9000,1 / 9000,0 portrait, else the first idle frame (anim 0).
/// Decodes the SFF but converts a single sprite to RGBA, so it is far cheaper than a full character load and
/// the result can outlive the (size-capped) `CharDef` cache.
pub fn load_mugen_portrait(vfs: &Vfs, def_path: &str) -> Option<Sprite> {
    let def_path = norm_path(def_path);
    let base = def_path.rsplit_once('/').map(|(b, _)| b.to_string()).unwrap_or_default();
    let secs = parse_ini(&text(vfs.get(&def_path)?));
    let files = find_section(&secs, "files")?;
    let fget = |k: &str| files.get(k).map(|s| s.to_string());
    let sff = decode_sff(resolve(vfs, &base, &fget("sprite")?)?).ok()?;
    let find = |g: i32, i: i32| sff.sprites.iter().find(|s| s.group == g && s.index == i && s.width > 0 && s.height > 0);
    let idle = || {
        let air = parse_air(&text(resolve(vfs, &base, &fget("anim")?)?));
        let fr = air.get(&0)?.frames.first()?.clone();
        find(fr.group, fr.index)
    };
    let act = fget("pal1").and_then(|p| resolve(vfs, &base, &p)).filter(|a| a.len() >= 768).map(decode_act);
    let fallback = act.or_else(|| sff.palettes.first().map(|p| p.data.to_vec()));
    // first candidate that actually has visible pixels (some packs ship blank 9000,x placeholders)
    let picked = [find(9000, 1), find(9000, 0), idle()].into_iter().flatten().find_map(|spr| {
        let use_act = spr.group != 9000 && sff.version == 1 || spr.palette.is_none() && spr.rgba.is_none();
        let rgba = sprite_to_rgba(spr, if use_act { fallback.as_deref() } else { None });
        if rgba.chunks_exact(4).all(|p| p[3] == 0) { return None; }
        Some(Sprite { image: Some(ImageData::new(spr.width, spr.height, rgba)), width: spr.width as f32, height: spr.height as f32, axis_x: spr.axis_x as f32, axis_y: spr.axis_y as f32, scale: 1.0 })
    });
    picked
}
