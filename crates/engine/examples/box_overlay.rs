use ume_engine::native::{compile_native, NativeChar};

const CELL: usize = 360;
const PX: f32 = 2.2;

fn rect(buf: &mut [u8], w: usize, b: [f32; 4], origin: (f32, f32), rgb: [u8; 3]) {
    let to = |v: f32, o: f32| (o + v * PX) as i64;
    let (x0, y0, x1, y1) = (
        to(b[0], origin.0),
        to(b[1], origin.1),
        to(b[2], origin.0),
        to(b[3], origin.1),
    );
    let mut put = |x: i64, y: i64| {
        if x >= 0
            && y >= 0
            && (x as usize) < w
            && (y as usize) < CELL * (buf.len() / (w * 4 * CELL))
        {
            let i = (y as usize * w + x as usize) * 4;
            buf[i..i + 4].copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
        }
    };
    for x in x0..=x1 {
        put(x, y0);
        put(x, y1);
    }
    for y in y0..=y1 {
        put(x0, y);
        put(x1, y);
    }
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let nc: NativeChar = serde_json::from_str(
        &std::fs::read_to_string(format!("assets/chars/{}.json", a[1])).unwrap(),
    )
    .unwrap();
    let def = compile_native(&nc, 1, true);
    let mut picks = vec![];
    let mut nos: Vec<_> = def.anims.keys().copied().collect();
    nos.sort();
    for no in nos {
        for f in &def.anims[&no].frames {
            if !f.clsn1.is_empty()
                && f.clsn1 != f.clsn2
                && picks.len() < 8
                && (no < 1000 || no > 1000)
            {
                picks.push(f.clone());
                break;
            }
        }
    }
    let cols = 4;
    let rows = picks.len().div_ceil(cols);
    let w = CELL * cols;
    let mut buf = vec![30u8; w * CELL * rows * 4];
    for px in buf.chunks_mut(4) {
        px[3] = 255;
        px[0] = 40;
        px[1] = 44;
        px[2] = 52;
    }
    for (n, f) in picks.iter().enumerate() {
        let (cx, cy) = ((n % cols) * CELL, (n / cols) * CELL);
        let origin = (cx as f32 + CELL as f32 * 0.4, cy as f32 + CELL as f32 * 0.9);
        let s = &def.sprites[&(f.group, f.index)];
        if let Some(img) = &s.image {
            let unit = ume_engine::native::rig::art_res() * s.scale;
            for y in 0..CELL {
                for x in 0..CELL {
                    let (ux, uy) = ((cx + x) as f32 - origin.0, (cy + y) as f32 - origin.1);
                    let (sx, sy) = (
                        (ux / PX - f.x + s.axis_x * s.scale) / unit,
                        (uy / PX - f.y + s.axis_y * s.scale) / unit,
                    );
                    if sx < 0.0 || sy < 0.0 || sx >= img.w as f32 || sy >= img.h as f32 {
                        continue;
                    }
                    let i = (sy as usize * img.w as usize + sx as usize) * 4;
                    let al = img.rgba[i + 3] as u32;
                    if al < 40 {
                        continue;
                    }
                    let o = ((cy + y) * w + cx + x) * 4;
                    for c in 0..3 {
                        buf[o + c] = ((img.rgba[i + c] as u32 * al
                            + buf[o + c] as u32 * (255 - al))
                            / 255) as u8;
                    }
                }
            }
        }
        for b in &f.clsn2 {
            rect(&mut buf, w, *b, origin, [60, 140, 255]);
        }
        for b in &f.clsn1 {
            rect(&mut buf, w, *b, origin, [255, 50, 50]);
        }
    }
    tiny_skia::Pixmap::from_vec(
        buf,
        tiny_skia::IntSize::from_wh(w as u32, (CELL * rows) as u32).unwrap(),
    )
    .unwrap()
    .save_png(&a[2])
    .unwrap();
}
