use crate::ini::{find_section, nums, parse_ini, unquote};
use crate::pcx::decode_pcx;
use crate::vfs::text;
use crate::Reader;
use std::collections::BTreeMap;

const SIGNATURE: &[u8; 12] = b"ElecbyteFnt\0";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FontKind {
    Fixed,
    Variable,
    Bitmap,
    TrueType,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Glyph {
    pub x: u32,
    pub w: u32,
}

#[derive(Clone, Debug)]
pub struct FontImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
    pub palette: Option<Vec<u8>>,
}

#[derive(Clone, Debug)]
pub struct Font {
    pub kind: FontKind,
    pub size: [i32; 2],
    pub spacing: [i32; 2],
    pub offset: [i32; 2],
    pub colors: i32,
    pub glyphs: BTreeMap<char, Glyph>,
    pub image: Option<FontImage>,
    pub file: Option<String>,
}

impl Font {
    pub fn glyph(&self, c: char) -> Option<Glyph> {
        self.glyphs.get(&c).copied().or_else(|| match self.kind {
            FontKind::Bitmap | FontKind::TrueType => Some(Glyph { x: c as u32, w: self.size[0].max(0) as u32 }),
            _ => None,
        })
    }

    pub fn text_width(&self, s: &str) -> i32 {
        let mut w = 0i32;
        let mut n = 0usize;
        for c in s.chars() {
            let g = self.glyph(c).or_else(|| self.glyph(' '));
            w = w.saturating_add(g.map(|g| g.w.min(i32::MAX as u32) as i32).unwrap_or(self.size[0]));
            n = n.saturating_add(1);
        }
        let spacing_count = n.saturating_sub(1).min(i32::MAX as usize) as i32;
        w.saturating_add(self.spacing[0].saturating_mul(spacing_count))
    }

    pub fn palette_bank(&self, bank: i32) -> Option<Vec<u8>> {
        let pal = self.image.as_ref()?.palette.as_ref()?;
        if self.colors <= 0 || bank <= 0 {
            return Some(pal.clone());
        }
        let colors = self.colors as usize;
        let start = 256usize.checked_sub(colors * (bank as usize + 1))?;
        let mut out = pal.clone();
        let src = 256 - colors;
        for i in 0..colors {
            out[(src + i) * 4..(src + i) * 4 + 4].copy_from_slice(&pal[(start + i) * 4..(start + i) * 4 + 4]);
        }
        Some(out)
    }
}

pub fn decode_fnt(buf: &[u8]) -> Result<Font, String> {
    if buf.len() >= 12 && &buf[..12] == SIGNATURE {
        decode_fnt_v1(buf)
    } else {
        decode_fnt_v2(&text(buf))
    }
}

fn pair(v: Option<&str>, d: [i32; 2]) -> [i32; 2] {
    let n = nums(v, &[d[0] as f32, d[1] as f32]);
    [n[0] as i32, n.get(1).map(|&x| x as i32).unwrap_or(d[1])]
}

fn parse_char(tok: &str) -> Option<char> {
    let lower = tok.to_ascii_lowercase();
    if tok.chars().count() > 1 {
        if let Some(h) = lower.strip_prefix("0x") {
            return u32::from_str_radix(h, 16).ok().and_then(char::from_u32);
        }
    }
    tok.chars().next()
}

fn decode_fnt_v1(buf: &[u8]) -> Result<Font, String> {
    let r = Reader(buf);
    let (pcx_off, pcx_len) = (r.u32(16) as usize, r.u32(20) as usize);
    let (txt_off, txt_len) = (r.u32(24) as usize, r.u32(28) as usize);
    let pcx = buf.get(pcx_off..pcx_off.saturating_add(pcx_len)).ok_or("FNT: PCX block out of range")?;
    let txt_end = txt_off.saturating_add(txt_len).min(buf.len());
    let txt = buf.get(txt_off..txt_end).ok_or("FNT: text block out of range")?;
    let img = decode_pcx(pcx, true).map_err(|e| format!("FNT: {e}"))?;
    let secs = parse_ini(&text(txt));
    let def = find_section(&secs, "def");
    let get = |k: &str| def.and_then(|s| s.get(k));
    let size = pair(get("size"), [8, 8]);
    let kind = match get("type").map(|t| unquote(t).to_ascii_lowercase()) {
        Some(t) if t == "fixed" => FontKind::Fixed,
        _ => FontKind::Variable,
    };
    let mut glyphs = BTreeMap::new();
    if let Some(map) = find_section(&secs, "map") {
        let mut slot = 0u32;
        for line in &map.lines {
            let toks: Vec<&str> = line.split_whitespace().collect();
            let Some(c) = toks.first().and_then(|t| parse_char(t)) else { continue };
            let glyph = match kind {
                FontKind::Fixed => {
                    if let Some(i) = toks.get(1).and_then(|t| t.parse::<u32>().ok()) {
                        slot = i;
                    }
                    Glyph { x: slot.saturating_mul(size[0].max(0) as u32), w: size[0].max(0) as u32 }
                }
                _ => {
                    let x = toks.get(1).and_then(|t| t.parse::<u32>().ok()).unwrap_or(0);
                    let w = toks.get(2).and_then(|t| t.parse::<u32>().ok()).unwrap_or(size[0].max(0) as u32);
                    Glyph { x, w }
                }
            };
            glyphs.insert(c, glyph);
            slot += 1;
        }
    }
    Ok(Font {
        kind,
        size,
        spacing: pair(get("spacing"), [0, 0]),
        offset: pair(get("offset"), [0, 0]),
        colors: nums(get("colors"), &[0.0])[0] as i32,
        glyphs,
        image: Some(FontImage { width: img.width, height: img.height, pixels: img.pixels, palette: img.palette }),
        file: None,
    })
}

fn decode_fnt_v2(src: &str) -> Result<Font, String> {
    let secs = parse_ini(src);
    let def = find_section(&secs, "def").ok_or("FNT: missing [Def] section")?;
    let kind = match def.get("type").map(|t| unquote(t).to_ascii_lowercase()) {
        Some(t) if t == "truetype" => FontKind::TrueType,
        _ => FontKind::Bitmap,
    };
    Ok(Font {
        kind,
        size: pair(def.get("size"), [8, 8]),
        spacing: pair(def.get("spacing"), [0, 0]),
        offset: pair(def.get("offset"), [0, 0]),
        colors: 0,
        glyphs: BTreeMap::new(),
        image: None,
        file: def.get("file").map(unquote).filter(|f| !f.is_empty()),
    })
}
