//! Renders the first frame of chosen animations of a MUGEN character with the palette the engine resolves.
//! Usage: cargo run --release -p ume-engine --example indexed_sheet -- <char_dir> <out.png> [pal_no] [anim...]
use tiny_skia::{Pixmap, PixmapPaint, Transform};
use ume_engine::loader::load_mugen_char;
use ume_formats::vfs::{add_dir, find_char_defs, Vfs};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let mut vfs = Vfs::new();
    add_dir(&mut vfs, std::path::Path::new(&a[1]), "c").unwrap();
    let def = find_char_defs(&vfs).remove(0);
    let pal_no: i32 = a.get(3).and_then(|s| s.parse().ok()).unwrap_or(1);
    let d = load_mugen_char(&vfs, &def, pal_no, true).unwrap();
    let mut anims: Vec<i32> = a.iter().skip(4).filter_map(|s| s.parse().ok()).collect();
    if anims.is_empty() { anims = vec![0, 5, 200, 400, 1000, 3000, 3050, 5000]; }
    let mut sheet = Pixmap::new(1800, 700).unwrap();
    sheet.fill(tiny_skia::Color::from_rgba8(70, 70, 70, 255));
    let (mut x, mut y, mut rh) = (4i32, 4i32, 0i32);
    for no in anims {
        let Some(an) = d.anims.get(&no) else { continue };
        for fr in an.frames.iter().take(2) {
            let key = (fr.group, fr.index);
            let Some(ix) = d.indexed.get(&key) else { continue };
            let pal = ix.palette.clone().or_else(|| d.palettes.get(&ix.pal).cloned());
            let Some(pal) = pal else { continue };
            let mut rgba = vec![0u8; ix.pixels.len() * 4];
            for (o, &c) in rgba.chunks_exact_mut(4).zip(ix.pixels.iter()) {
                if c != 0 { let c = c as usize * 4; o[..3].copy_from_slice(&pal[c..c + 3]); o[3] = 255; }
            }
            let mut pm = Pixmap::new(ix.w, ix.h).unwrap();
            for (dst, s) in pm.data_mut().chunks_exact_mut(4).zip(rgba.chunks_exact(4)) {
                dst.copy_from_slice(&[s[0], s[1], s[2], s[3]]);
            }
            let (w, h) = (ix.w as i32, ix.h as i32);
            if x + w > 1800 { x = 4; y += rh + 4; rh = 0; }
            if y + h > 700 { break; }
            sheet.draw_pixmap(x, y, pm.as_ref(), &PixmapPaint::default(), Transform::identity(), None);
            x += w + 4; rh = rh.max(h);
        }
    }
    sheet.save_png(&a[2]).unwrap();
}
