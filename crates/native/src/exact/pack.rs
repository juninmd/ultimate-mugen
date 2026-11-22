use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;
use ume_formats::png::decode_png;
use ume_formats::png_encode::{encode_palette_png, encode_rgba_png};
use ume_formats::sff::{DecodedSff, DecodedSprite, Palette};

pub const META: &str = "meta.json";

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PackMeta {
    pub sff: SffMeta,
    #[serde(default)]
    pub sounds: Vec<[i32; 2]>,
    #[serde(default)]
    pub acts: Vec<usize>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SffMeta {
    pub version: u8,
    #[serde(default)]
    pub palettes: Vec<[i32; 2]>,
    pub sprites: Vec<SpriteMeta>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SpriteMeta {
    pub g: i32,
    pub i: i32,
    pub axis: [i32; 2],
    pub size: [u32; 2],
    #[serde(default = "no_palette", skip_serializing_if = "is_no_palette")]
    pub pal: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub rgba: bool,
    #[serde(default, rename = "noPalette", skip_serializing_if = "is_false")]
    pub no_palette: bool,
}

fn no_palette() -> i32 {
    -1
}

fn is_no_palette(v: &i32) -> bool {
    *v == -1
}

fn is_false(v: &bool) -> bool {
    !*v
}

pub fn act_name(k: usize) -> String {
    format!("pal{k}.act")
}

pub fn palette_name(k: usize) -> String {
    format!("sffpal_{k}.png")
}

pub fn sound_name(g: i32, i: i32) -> String {
    format!("snd_{g}_{i}.wav")
}

fn sprite_png(s: &DecodedSprite) -> Option<(Vec<u8>, bool, bool)> {
    let (w, h) = (s.width as usize, s.height as usize);
    if w == 0 || h == 0 {
        return None;
    }
    if let Some(px) = &s.indexed {
        let pal: Vec<[u8; 4]> = match &s.palette {
            Some(p) => (0..256).map(|k| std::array::from_fn(|c| p.get(k * 4 + c).copied().unwrap_or(0))).collect(),
            None => vec![[0, 0, 0, 255]; 256],
        };
        return Some((encode_palette_png(w, h, px, &pal, 9), false, s.palette.is_none()));
    }
    s.rgba.as_ref().map(|px| (encode_rgba_png(w, h, px, 9), true, false))
}

pub fn encode_pack(sff: &DecodedSff, sounds: &HashMap<(i32, i32), Vec<u8>>, acts: &[Option<Vec<u8>>]) -> (PackMeta, Vec<(String, Vec<u8>)>) {
    let mut files: Vec<(String, Vec<u8>)> = vec![];
    let mut by_bytes: HashMap<Vec<u8>, String> = HashMap::new();
    let mut seen = HashSet::new();
    let encoded = ume_formats::par::map(&sff.sprites, |s| sprite_png(s));
    let sprites = sff
        .sprites
        .iter()
        .zip(encoded)
        .map(|(s, png)| {
            let mut meta = SpriteMeta { g: s.group, i: s.index, axis: [s.axis_x, s.axis_y], size: [s.width, s.height], pal: s.palette_index, ..Default::default() };
            if let (true, Some((bytes, rgba, no_palette))) = (seen.insert((s.group, s.index)), png) {
                let name = by_bytes.entry(bytes.clone()).or_insert_with(|| {
                    let name = format!("s{}_{}.png", s.group, s.index);
                    files.push((name.clone(), bytes));
                    name
                });
                meta.file = Some(name.clone());
                meta.rgba = rgba;
                meta.no_palette = no_palette;
            }
            meta
        })
        .collect();
    for (k, p) in sff.palettes.iter().enumerate() {
        let mut rgba = p.data.to_vec();
        rgba.resize(1024, 0);
        files.push((palette_name(k), encode_rgba_png(16, 16, &rgba, 9)));
    }
    let mut keys: Vec<_> = sounds.keys().copied().collect();
    keys.sort_unstable();
    for &(g, i) in &keys {
        files.push((sound_name(g, i), sounds[&(g, i)].clone()));
    }
    let mut act_list = vec![];
    for (k, act) in acts.iter().enumerate() {
        if let Some(bytes) = act {
            files.push((act_name(k + 1), bytes.clone()));
            act_list.push(k + 1);
        }
    }
    let meta = PackMeta {
        sff: SffMeta { version: sff.version, palettes: sff.palettes.iter().map(|p| [p.group, p.index]).collect(), sprites },
        sounds: keys.iter().map(|&(g, i)| [g, i]).collect(),
        acts: act_list,
    };
    (meta, files)
}

pub struct DecodedPack {
    pub sff: DecodedSff,
    pub sounds: HashMap<(i32, i32), Vec<u8>>,
    pub acts: Vec<Option<Vec<u8>>>,
}

fn decode_sprite(m: &SpriteMeta, bytes: Option<&[u8]>) -> DecodedSprite {
    let mut s = DecodedSprite { group: m.g, index: m.i, axis_x: m.axis[0], axis_y: m.axis[1], width: m.size[0], height: m.size[1], indexed: None, rgba: None, palette: None, palette_index: m.pal };
    let Some(png) = bytes.and_then(|b| decode_png(b).ok()) else { return s };
    if m.rgba {
        s.rgba = png.rgba.map(Arc::new);
    } else {
        s.indexed = png.indexed.map(Arc::new);
        s.palette = if m.no_palette { None } else { png.palette.map(Arc::new) };
    }
    s
}

pub fn decode_pack(meta: &PackMeta, read: &(dyn Fn(&str) -> Option<Arc<Vec<u8>>> + Sync), wanted: Option<&HashSet<(i32, i32)>>) -> DecodedPack {
    let palettes: Vec<Palette> = meta
        .sff
        .palettes
        .iter()
        .enumerate()
        .map(|(k, [group, index])| Palette { group: *group, index: *index, data: Arc::new(read(&palette_name(k)).and_then(|b| decode_png(&b).ok()?.rgba).unwrap_or_else(|| vec![0; 1024])) })
        .collect();
    let mut cache: BTreeMap<&str, Option<Arc<Vec<u8>>>> = BTreeMap::new();
    for m in &meta.sff.sprites {
        if let Some(f) = &m.file {
            if wanted.is_none_or(|w| w.contains(&(m.g, m.i))) {
                cache.entry(f.as_str()).or_insert_with(|| read(f));
            }
        }
    }
    let sprites = ume_formats::par::map(&meta.sff.sprites, |m| {
        let bytes = m.file.as_deref().and_then(|f| cache.get(f)).and_then(|b| b.as_deref()).map(|b| b.as_slice());
        decode_sprite(m, bytes)
    });
    let sounds = meta.sounds.iter().filter_map(|&[g, i]| Some(((g, i), read(&sound_name(g, i))?.to_vec()))).collect();
    let mut acts = vec![None; 12];
    for &k in &meta.acts {
        if let Some(slot) = k.checked_sub(1).and_then(|i| acts.get_mut(i)) {
            *slot = read(&act_name(k)).map(|b| b.to_vec());
        }
    }
    DecodedPack { sff: DecodedSff { version: meta.sff.version, sprites, palettes }, sounds, acts }
}
