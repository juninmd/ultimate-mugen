use super::image::Rgba;
use super::regions::Mask;
use std::collections::HashMap;

pub struct MaskOptions {
    pub background: Option<[u8; 3]>,
    pub tolerance: i32,
    pub ignore: Vec<[u8; 3]>,
    pub alpha_threshold: u8,
}

impl Default for MaskOptions {
    fn default() -> Self {
        MaskOptions { background: None, tolerance: 8, ignore: Vec::new(), alpha_threshold: 16 }
    }
}

pub fn parse_color(text: &str) -> Result<[u8; 3], String> {
    let hex = text.trim_start_matches('#');
    if hex.len() != 6 {
        return Err(format!("color must be #rrggbb, got {text}"));
    }
    let channel = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).map_err(|e| format!("{text}: {e}"));
    Ok([channel(0)?, channel(2)?, channel(4)?])
}

fn distance(pixel: [u8; 4], color: [u8; 3]) -> i32 {
    (0..3).map(|i| (pixel[i] as i32 - color[i] as i32).abs()).max().unwrap_or(0)
}

fn border_color(image: &Rgba) -> [u8; 3] {
    let mut counts: HashMap<[u8; 3], usize> = HashMap::new();
    let mut count = |x: usize, y: usize| {
        let p = image.pixel(x, y);
        *counts.entry([p[0], p[1], p[2]]).or_default() += 1;
    };
    for x in 0..image.width {
        count(x, 0);
        count(x, image.height - 1);
    }
    for y in 0..image.height {
        count(0, y);
        count(image.width - 1, y);
    }
    counts.into_iter().max_by_key(|(color, n)| (*n, std::cmp::Reverse(*color))).map(|(c, _)| c).unwrap_or([0, 0, 0])
}

pub fn foreground_mask(image: &Rgba, options: &MaskOptions) -> Mask {
    let translucent = image.data.chunks_exact(4).filter(|p| p[3] < 255).count();
    let use_alpha = options.background.is_none() && translucent * 50 > image.width * image.height;
    let background = options.background.unwrap_or_else(|| border_color(image));
    let mut mask = Mask::new(image.width, image.height);
    for y in 0..image.height {
        for x in 0..image.width {
            let pixel = image.pixel(x, y);
            let solid = if use_alpha {
                pixel[3] > options.alpha_threshold
            } else {
                distance(pixel, background) > options.tolerance && pixel[3] > options.alpha_threshold
            };
            let ignored = options.ignore.iter().any(|c| distance(pixel, *c) <= options.tolerance);
            mask.set(x, y, solid && !ignored);
        }
    }
    mask
}
