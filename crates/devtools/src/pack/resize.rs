pub struct Image {
    pub width: usize,
    pub height: usize,
    pub rgba: Vec<u8>,
}

fn lanczos(x: f64) -> f64 {
    if x == 0.0 {
        return 1.0;
    }
    if x.abs() >= 3.0 {
        return 0.0;
    }
    let px = std::f64::consts::PI * x;
    3.0 * px.sin() * (px / 3.0).sin() / (px * px)
}

struct Weights {
    first: Vec<usize>,
    values: Vec<Vec<f32>>,
}

fn weights(input: usize, output: usize) -> Weights {
    let scale = input as f64 / output as f64;
    let filter_scale = scale.max(1.0);
    let support = 3.0 * filter_scale;
    let mut first = Vec::with_capacity(output);
    let mut values = Vec::with_capacity(output);
    for out in 0..output {
        let center = (out as f64 + 0.5) * scale;
        let low = ((center - support + 0.5).floor().max(0.0)) as usize;
        let high = (((center + support + 0.5).floor()) as usize).min(input);
        let mut row: Vec<f64> = (low..high).map(|i| lanczos((i as f64 - center + 0.5) / filter_scale)).collect();
        let total: f64 = row.iter().sum();
        if total != 0.0 {
            row.iter_mut().for_each(|w| *w /= total);
        }
        first.push(low);
        values.push(row.into_iter().map(|w| w as f32).collect());
    }
    Weights { first, values }
}

fn premultiply(image: &Image) -> Vec<f32> {
    image.rgba.chunks_exact(4).flat_map(|p| {
        let a = p[3] as f32 / 255.0;
        [p[0] as f32 * a, p[1] as f32 * a, p[2] as f32 * a, p[3] as f32]
    }).collect()
}

fn pass(source: &[f32], width: usize, height: usize, new_width: usize) -> Vec<f32> {
    let w = weights(width, new_width);
    let mut out = vec![0f32; new_width * height * 4];
    for y in 0..height {
        for x in 0..new_width {
            let mut acc = [0f32; 4];
            for (k, weight) in w.values[x].iter().enumerate() {
                let i = (y * width + w.first[x] + k) * 4;
                for c in 0..4 {
                    acc[c] += source[i + c] * weight;
                }
            }
            out[(y * new_width + x) * 4..(y * new_width + x) * 4 + 4].copy_from_slice(&acc);
        }
    }
    out
}

fn transpose(source: &[f32], width: usize, height: usize) -> Vec<f32> {
    let mut out = vec![0f32; source.len()];
    for y in 0..height {
        for x in 0..width {
            out[(x * height + y) * 4..(x * height + y) * 4 + 4].copy_from_slice(&source[(y * width + x) * 4..(y * width + x) * 4 + 4]);
        }
    }
    out
}

pub fn lanczos_resize(image: &Image, new_width: usize, new_height: usize) -> Image {
    let horizontal = pass(&premultiply(image), image.width, image.height, new_width);
    let turned = transpose(&horizontal, new_width, image.height);
    let vertical = pass(&turned, image.height, new_width, new_height);
    let result = transpose(&vertical, new_height, new_width);
    let mut rgba = Vec::with_capacity(new_width * new_height * 4);
    for p in result.chunks_exact(4) {
        let alpha = p[3].clamp(0.0, 255.0);
        let un = if alpha > 0.0 { 255.0 / alpha } else { 0.0 };
        rgba.extend_from_slice(&[(p[0] * un).round().clamp(0.0, 255.0) as u8, (p[1] * un).round().clamp(0.0, 255.0) as u8, (p[2] * un).round().clamp(0.0, 255.0) as u8, alpha.round() as u8]);
    }
    Image { width: new_width, height: new_height, rgba }
}
