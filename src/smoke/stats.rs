pub struct ImageStats {
    pub mean: f32,
    pub deviation: f32,
}

impl ImageStats {
    pub fn looks_blank(&self) -> bool {
        self.mean < 6.0 || self.deviation < 4.0
    }

    pub fn describe(&self) -> String {
        format!("luminance mean {:.1}, deviation {:.1}", self.mean, self.deviation)
    }
}

pub fn of_png(bytes: &[u8]) -> Result<ImageStats, String> {
    let png = ume_formats::png::decode_png(bytes)?;
    let rgba = match (&png.rgba, &png.indexed, &png.palette) {
        (Some(rgba), _, _) => rgba.clone(),
        _ => ume_formats::png::png_indexed_to_rgba(&png),
    };
    let pixels = rgba.chunks_exact(4).step_by(7);
    let (mut sum, mut sum2, mut n) = (0.0f64, 0.0f64, 0.0f64);
    for p in pixels {
        let l = (p[0] as f64 + p[1] as f64 + p[2] as f64) / 3.0;
        sum += l;
        sum2 += l * l;
        n += 1.0;
    }
    if n == 0.0 {
        return Err("empty image".into());
    }
    let mean = sum / n;
    Ok(ImageStats { mean: mean as f32, deviation: (sum2 / n - mean * mean).max(0.0).sqrt() as f32 })
}
