//! Build a CharDef from a MUGEN character inside a VFS.
use crate::chardef::{
    default_constants, CharDef, ImageData, IndexedSprite, MoveInfo, Source, Sprite,
};
use crate::cns::{parse_cmd, parse_constants, parse_states, scale_default_constants};
use crate::commands::parse_command;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use ume_formats::act::decode_act;
use ume_formats::air::parse_air;
use ume_formats::ini::{find_section, parse_float, parse_ini, unquote_str};
use ume_formats::sff::{decode_sff, decode_sff_only, sprite_to_rgba};
use ume_formats::snd::decode_snd;
use ume_formats::vfs::{norm_path, resolve, text, Vfs};

/// `localcoord` -> (largura, altura). Aceita o formato canonico `320,240`,
/// o valor unico usado por varios chars de fullgames (`400` vira `400x240`)
/// e o campo ausente/invalido (320x240).
fn local_size(lc: Option<&str>) -> (f32, f32) {
    let Some(lc) = lc else { return (320.0, 240.0); };
    let lc = lc.trim();
    if lc.is_empty() { return (320.0, 240.0); };
    if let Some((w_s, h_s)) = lc.split_once(',') {
        let w = parse_float(w_s).map(|v| v as f32).filter(|w| *w > 0.0).unwrap_or(320.0);
        let h = parse_float(h_s).map(|v| v as f32).filter(|h| *h > 0.0).unwrap_or(240.0);
        (w, h)
    } else {
        let w = parse_float(lc).map(|v| v as f32).filter(|w| *w > 0.0).unwrap_or(320.0);
        (w, 240.0)
    }
}

pub fn load_mugen_char(
    vfs: &Vfs,
    def_path: &str,
    pal_no: i32,
    with_images: bool,
) -> Result<CharDef, String> {
    let def_path = norm_path(def_path);
    let base = def_path
        .rsplit_once('/')
        .map(|(b, _)| b.to_string())
        .unwrap_or_default();
    let dt = text(vfs.get(&def_path).ok_or("def not found")?);
    let secs = parse_ini(&dt);
    let info = find_section(&secs, "info");
    let files = find_section(&secs, "files");
    let fget = |k: &str| files.and_then(|s| s.get(k));
    let name = info
        .and_then(|s| s.get("name"))
        .map(unquote_str)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| def_path.clone());
    let display = info
        .and_then(|s| s.get("displayname"))
        .map(unquote_str)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| name.clone());
    let author = info
        .and_then(|s| s.get("author"))
        .map(unquote_str)
        .unwrap_or_default()
        .to_string();
    let (local_w, local_h) = local_size(info.and_then(|s| s.get("localcoord")));

    let mut constants = default_constants();
    let mut provided = HashSet::new();
    let mut states = HashMap::new();
    let mut spans = crate::diagnostics::SourceSpans::default();

    let sprite_buf = fget("sprite").and_then(|p| resolve(vfs, &base, p));
    let anim_buf = fget("anim").and_then(|p| resolve(vfs, &base, p));
    let snd_buf = fget("sound").and_then(|p| resolve(vfs, &base, p));

    // `stcommon` (normalmente common1.cns) traz os estados comuns E, em varios chars de
    // fullgames, o mapeamento comando -> ChangeState do [Statedef -1]. Sem ele o lutador
    // carrega e anda, mas nunca ataca. So vale o arquivo dentro da pasta do char: o
    // fallback global (`data/common1.cns`) aplicaria o common de um jogo a outro.
    let common_name = unquote_str(fget("stcommon").unwrap_or_default());
    if !common_name.is_empty()
        && !common_name.contains("..")
        && !common_name.contains('/')
        && !common_name.contains('\\')
    {
        if let Some(b) = vfs.get(&norm_path(&format!("{base}/{common_name}"))) {
            spans.track(common_name, || {
                crate::zss::parse_state_file(common_name, &text(b), &mut states, false)
            });
        }
    }
    let cns_name = fget("cns").unwrap_or_default();
    let cns_text = crate::perf::time("cns:read", || fget("cns").and_then(|c| resolve(vfs, &base, c)).map(text).unwrap_or_default());
    parse_constants(&cns_text, &mut constants, &mut provided);
    scale_default_constants(&mut constants, &provided, local_w as f64 / 320.0);
    crate::perf::time("cns:parse", || spans.track(cns_name, || parse_states(&cns_text, &mut states, true)));
    for k in ["st", "st0", "st1", "st2", "st3", "st4", "st5", "st6", "st7", "st8", "st9"] {
        let Some(fname) = fget(k) else { continue };
        if norm_path(unquote_str(fname)) == norm_path(unquote_str(cns_name)) {
            continue;
        }
        if let Some(b) = resolve(vfs, &base, fname) {
            spans.track(fname, || {
                crate::zss::parse_state_file(fname, &text(b), &mut states, true)
            });
        }
    }
    let cmd_name = fget("cmd").unwrap_or_default();
    let cmd_text = fget("cmd")
        .and_then(|c| resolve(vfs, &base, c))
        .map(text)
        .unwrap_or_default();
    let (mut commands, cmd_states) =
        spans.track(cmd_name, || parse_cmd(&cmd_text));
    for (id, sd) in cmd_states {
        if id < 0 && states.contains_key(&id) {
            if let Some(s) = states.get_mut(&id) {
                s.controllers.extend(sd.controllers)
            }
        } else {
            states.insert(id, sd);
        }
    }
    for (n, c) in [
        ("holdfwd", "/$F"),
        ("holdback", "/$B"),
        ("holdup", "/$U"),
        ("holddown", "/$D"),
    ] {
        if !commands.iter().any(|x| x.name == n) {
            commands.push(parse_command(n, c, 1, 1));
        }
    }
    let (sff_opt, anims, sounds) = {
        #[cfg(not(target_arch = "wasm32"))]
        {
            std::thread::scope(|sc| {
                let h_sff = sc.spawn(move || {
                    sprite_buf.map(|buf| crate::perf::time("sff:decode", || decode_sff(buf)))
                });
                let h_anim = sc.spawn(move || {
                    crate::perf::time("air:parse", || {
                        anim_buf.map(|b| parse_air(&text(b))).unwrap_or_default()
                    })
                });
                let h_snd = sc.spawn(move || {
                    crate::perf::time("snd:decode", || snd_buf.map(decode_snd).unwrap_or_default())
                });
                (
                    h_sff.join().unwrap(),
                    h_anim.join().unwrap(),
                    h_snd.join().unwrap(),
                )
            })
        }
        #[cfg(target_arch = "wasm32")]
        {
            let sff = sprite_buf.map(|buf| crate::perf::time("sff:decode", || decode_sff(buf)));
            let anims = crate::perf::time("air:parse", || {
                anim_buf.map(|b| parse_air(&text(b))).unwrap_or_default()
            });
            let sounds = crate::perf::time("snd:decode", || {
                snd_buf.map(decode_snd).unwrap_or_default()
            });
            (sff, anims, sounds)
        }
    };

    let mut sprites = HashMap::new();
    let mut palettes = HashMap::new();
    let mut indexed = HashMap::new();
    if let Some(sff) = match sff_opt {
        Some(res) => Some(res?),
        None => None,
    } {
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
        let mut pal_kbuf = [0u8; 16];
        let pal_k = {
            use std::io::Write;
            let mut cur = std::io::Cursor::new(&mut pal_kbuf[..]);
            let _ = write!(cur, "pal{pal_no}");
            let pos = cur.position() as usize;
            std::str::from_utf8(&pal_kbuf[..pos]).unwrap_or("")
        };
        if sff.version == 1 {
            if let Some(a) = fget(pal_k).and_then(|p| resolve(vfs, &base, p)) {
                if a.len() >= 768 {
                    over = Some(decode_act(a));
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
                fget(pal_k).and_then(|p| resolve(vfs, &base, p))
            {
                if a.len() >= 768 {
                    over = Some(decode_act(a));
                }
            }
        }
        // v1 sprites without an embedded PCX palette rely on the character's own palette (the .act / first SFF palette)
        let fallback: Option<Vec<u8>> = over
            .clone()
            .or_else(|| {
                fget("pal1")
                    .and_then(|p| resolve(vfs, &base, p))
                    .filter(|a| a.len() >= 768)
                    .map(decode_act)
            })
            .or_else(|| sff.palettes.first().map(|p| p.data.to_vec()));
        let shared_key = |s: &ume_formats::sff::DecodedSprite| -> (i32, i32) {
            if sff.version == 1 {
                return if s.palette_index == target || s.palette.is_none() {
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
            const PAL_KEYS: [&str; 12] = [
                "pal1", "pal2", "pal3", "pal4", "pal5", "pal6",
                "pal7", "pal8", "pal9", "pal10", "pal11", "pal12",
            ];
            for (i, &key) in PAL_KEYS.iter().enumerate() {
                let k = (i + 1) as i32;
                if palettes.contains_key(&(1, k)) {
                    continue;
                }
                if let Some(a) = fget(key)
                    .and_then(|p| resolve(vfs, &base, p))
                    .filter(|a| a.len() >= 768)
                {
                    palettes.insert((1, k), Arc::new(decode_act(a)));
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
            } else if over.is_some() && s.palette_index == target && s.group != 9000 {
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
            } else if over.is_some() && s.palette_index == target && s.group != 9000 {
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
    }
    // move list for the pause screen: every author-defined command with its input (internal hold/recovery helpers left out)
    let mut movelist: Vec<MoveInfo> = Vec::with_capacity(commands.len().min(64));
    for c in commands.iter().filter(|c| !c.raw.is_empty()) {
        let single = !c.raw.contains(',') && !c.raw.contains('+');
        if single {
            continue;
        }
        let b = c.name.as_bytes();
        let is_hold = b.len() >= 4 && b[..4].eq_ignore_ascii_case(b"hold");
        let is_rec = b.len() >= 8 && b[..8].eq_ignore_ascii_case(b"recovery");
        if is_hold || is_rec || movelist.iter().any(|m| m.command == c.name) {
            continue;
        }
        movelist.push(MoveInfo {
            name: c.name.replace('_', " "),
            input: c.raw.clone(),
            command: c.name.clone(),
        });
    }
    let ai_commands = commands
        .iter()
        .filter(|c| {
            let b = c.name.as_bytes();
            let is_hold = b.len() >= 4 && b[..4].eq_ignore_ascii_case(b"hold");
            let is_rec = b.len() >= 8 && b[..8].eq_ignore_ascii_case(b"recovery");
            let is_dir = ["fwd", "back", "up", "down", "ff", "bb"]
                .iter()
                .any(|d| d.eq_ignore_ascii_case(&c.name));
            !is_hold && !is_rec && !is_dir
        })
        .map(|c| c.name.clone())
        .collect();
    let mut d = CharDef::empty(&format!("mugen:{def_path}"));
    d.name = name;
    d.display_name = display;
    d.author = author;
    d.source = Source::Mugen;
    d.local_w = local_w;
    d.local_h = local_h;
    d.mugen_version = crate::chardef::parse_mugen_version(info.and_then(|s| s.get("mugenversion")));
    d.constants = constants;
    d.portrait = sprites.get(&(9000, 0)).cloned();
    d.sprites = sprites;
    d.anims = anims;
    d.commands = commands;
    d.states = states;
    d.sounds = sounds;
    d.ai_commands = ai_commands;
    d.movelist = movelist;
    d.series = "M.U.G.E.N".into();
    d.palettes = palettes;
    d.indexed = indexed;
    d.source_spans = spans;
    report_diagnostics(&d, &def_path);
    let _ = Arc::new(0);
    Ok(d)
}

fn report_diagnostics(d: &CharDef, def_path: &str) {
    if cfg!(target_arch = "wasm32") || std::env::var_os("UME_QUIET_DIAGNOSTICS").is_some() {
        return;
    }
    eprint!(
        "{}",
        d.diagnostics()
            .report(def_path, std::env::var_os("TRACE").is_some())
    );
}

/// Select-screen thumbnail: the 9000,1 / 9000,0 portrait, else the first idle frame (anim 0).
/// Decodes the SFF but converts a single sprite to RGBA, so it is far cheaper than a full character load and
/// the result can outlive the (size-capped) `CharDef` cache.
pub fn load_mugen_portrait(vfs: &Vfs, def_path: &str) -> Option<Sprite> {
    let def_path = norm_path(def_path);
    let base = def_path
        .rsplit_once('/')
        .map(|(b, _)| b.to_string())
        .unwrap_or_default();
    let secs = parse_ini(&text(vfs.get(&def_path)?));
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

#[cfg(test)]
mod tests {
    use super::local_size;

    #[test]
    fn localcoord_accepts_two_values() {
        assert_eq!(local_size(Some("400, 300")), (400.0, 300.0));
    }

    #[test]
    fn localcoord_single_value_does_not_panic() {
        // chars de fullgames (KOF Zillion) usam `localcoord = 400`
        assert_eq!(local_size(Some("400")), (400.0, 240.0));
    }

    #[test]
    fn localcoord_missing_or_invalid_falls_back() {
        assert_eq!(local_size(None), (320.0, 240.0));
        assert_eq!(local_size(Some("")), (320.0, 240.0));
        assert_eq!(local_size(Some("0")), (320.0, 240.0));
    }
}
