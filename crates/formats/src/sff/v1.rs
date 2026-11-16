//! SFF v1 decoder (PCX subfiles). Split out of `sff.rs` so that file can stay
//! within its size budget; the logic is unchanged.

use super::{empty_sprite, DecodedSff, DecodedSprite, Palette, MAX_SPRITE_PIXELS};
use crate::pcx::{
    decode_pcx_rle, find_pcx_palette, raw_rows, read_pcx_header, read_rgb_palette, PCX_HEADER_SIZE,
};
use crate::Reader;
use std::collections::HashMap;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// SFF v1
// ---------------------------------------------------------------------------

struct V1Job<'a> {
    pcx: &'a [u8],
    h: crate::pcx::PcxHeader,
    same_pal: bool,
}

/// SFF v1 layout:
///   header: [0]sig(12) [12]ver(4) [16]numGroups u32 [20]numImages u32
///           [24]firstSubfileOffset u32 [28]subheaderSize u32 [32]paletteType u8
///   subfile: [0]next u32 [4]length u32 [8]axisX i16 [10]axisY i16 [12]group u16
///            [14]index u16 [16]linkIndex u16 [18]samePalette u8 [19]comment(13)
///            followed by PCX data (length bytes; 0 => linked sprite).
pub(super) fn decode_sff_v1(buf: &[u8], wanted: impl Fn(i32, i32) -> bool) -> DecodedSff {
    let r = Reader(buf);
    let count = (r.u32(20) as usize).min(buf.len() / 32 + 1);
    let mut ofs = r.u32(24) as usize;
    let mut palettes: Vec<Palette> = Vec::new();
    let mut pal_keys: HashMap<Arc<Vec<u8>>, usize> = HashMap::new();
    let mut prev: Option<(Option<Arc<Vec<u8>>>, i32)> = None;
    let mut prev_adjacent = false;

    let mut entries: Vec<(DecodedSprite, usize, bool)> = Vec::with_capacity(count);
    let mut jobs: Vec<V1Job> = Vec::with_capacity(count.min(8192));

    for _ in 0..count {
        if ofs == 0 || ofs.saturating_add(32) > buf.len() {
            break;
        }
        let next = r.u32(ofs) as usize;
        let length = r.u32(ofs + 4) as usize;
        let mut spr = empty_sprite(
            r.u16(ofs + 12) as i32,
            r.u16(ofs + 14) as i32,
            r.i16(ofs + 8) as i32,
            r.i16(ofs + 10) as i32,
        );
        let link = r.u16(ofs + 16) as usize;
        let same_pal = r.u8(ofs + 18) != 0;
        let data_start = ofs + 32;
        let keep = wanted(spr.group, spr.index);
        let mut has_data = false;

        if length > 0 && keep {
            let block_end = if next > data_start { next } else { data_start.saturating_add(length) }.min(buf.len());
            let pcx = &buf[data_start..block_end];
            if let Some(h) = read_pcx_header(pcx).ok().filter(|h| {
                (h.width as usize).saturating_mul(h.height as usize) <= MAX_SPRITE_PIXELS
            }) {
                spr.width = h.width;
                spr.height = h.height;
                has_data = true;
                jobs.push(V1Job { pcx, h, same_pal });
            }
        }
        entries.push((spr, link, has_data));
        ofs = next;
    }

    let decoded: Vec<(Arc<Vec<u8>>, usize)> = crate::par::map(&jobs, |j| {
        if j.h.encoding == 1 {
            let (px, end) = decode_pcx_rle(
                j.pcx,
                j.h.width,
                j.h.height,
                j.h.bytes_per_line,
                PCX_HEADER_SIZE,
                j.pcx.len(),
            );
            (Arc::new(px), end)
        } else {
            let px = raw_rows(
                j.pcx,
                j.h.width as usize,
                j.h.height as usize,
                j.h.bytes_per_line as usize,
            );
            let end = PCX_HEADER_SIZE + j.h.height as usize * j.h.bytes_per_line as usize;
            (Arc::new(px), end)
        }
    });

    let mut sprites: Vec<DecodedSprite> = Vec::with_capacity(entries.len());
    let mut job_iter = jobs.into_iter().zip(decoded);
    let mut next_job = job_iter.next();
    for (mut spr, link, has_data) in entries {
        let job = if has_data {
            let (j, (px, end)) = next_job.take().expect("sff v1 job order");
            next_job = job_iter.next();
            Some((j, px, end))
        } else {
            None
        };
        if let Some((j, px, end)) = job {
            let keep = wanted(spr.group, spr.index);
            if keep {
                spr.indexed = Some(px);
                let pcx = j.pcx;
                let mut pal: Option<Arc<Vec<u8>>> = None;
                let inherited = if j.same_pal && prev_adjacent { prev.clone() } else { None };
                match inherited {
                    Some((ppal, pidx)) => {
                        pal = ppal.clone();
                        spr.palette_index = pidx;
                    }
                    None => {
                        if let Some(po) = find_pcx_palette(pcx, end) {
                            pal = Some(Arc::new(read_rgb_palette(pcx, po)));
                        } else if prev_adjacent {
                            if let Some((ppal, pidx)) = &prev {
                                pal = ppal.clone();
                                spr.palette_index = *pidx;
                            }
                        }
                        if let Some(p) = &pal {
                            if spr.palette_index < 0 {
                                if let Some(&found) = pal_keys.get(p) {
                                    pal = Some(palettes[found].data.clone());
                                    spr.palette_index = found as i32;
                                } else {
                                    let idx = palettes.len();
                                    palettes.push(Palette {
                                        group: spr.group,
                                        index: spr.index,
                                        data: p.clone(),
                                    });
                                    pal_keys.insert(p.clone(), idx);
                                    spr.palette_index = idx as i32;
                                }
                            }
                        }
                    }
                }
                spr.palette = pal;
            }
        } else if wanted(spr.group, spr.index) {
            if let Some(src) = sprites.get(link) {
                spr.width = src.width;
                spr.height = src.height;
                spr.indexed = src.indexed.clone();
                spr.rgba = src.rgba.clone();
                spr.palette = src.palette.clone();
                spr.palette_index = src.palette_index;
            }
        }
        if spr.indexed.is_some() {
            prev = Some((spr.palette.clone(), spr.palette_index));
        }
        prev_adjacent = spr.indexed.is_some();
        sprites.push(spr);
    }

    DecodedSff {
        version: 1,
        sprites,
        palettes,
    }
}

