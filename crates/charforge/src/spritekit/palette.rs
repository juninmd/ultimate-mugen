use super::image::Rgba;
use std::collections::{BTreeMap, HashMap};

const TRANSPARENT_SLOT: [u8; 3] = [255, 0, 255];

fn opaque_colors(crops: &[Rgba]) -> BTreeMap<[u8; 3], u64> {
    let mut counts = BTreeMap::new();
    for crop in crops {
        for pixel in crop.data.chunks_exact(4).filter(|p| p[3] > 0) {
            *counts.entry([pixel[0], pixel[1], pixel[2]]).or_insert(0) += 1;
        }
    }
    counts
}

fn median_cut(colors: Vec<([u8; 3], u64)>, target: usize) -> Vec<[u8; 3]> {
    let mut boxes = vec![colors];
    while boxes.len() < target {
        let splittable = boxes.iter().enumerate().filter(|(_, b)| b.len() > 1).max_by_key(|(_, b)| {
            (0..3).map(|c| channel_range(b, c)).max().unwrap_or(0) as u64 * b.iter().map(|(_, n)| n).sum::<u64>()
        });
        let Some((index, _)) = splittable else { break };
        let mut chosen = boxes.swap_remove(index);
        let channel = (0..3).max_by_key(|&c| channel_range(&chosen, c)).unwrap_or(0);
        chosen.sort_by_key(|(color, _)| color[channel]);
        let half = chosen.len() / 2;
        let upper = chosen.split_off(half);
        boxes.push(chosen);
        boxes.push(upper);
    }
    boxes.iter().map(|b| average(b)).collect()
}

fn channel_range(colors: &[([u8; 3], u64)], channel: usize) -> u8 {
    let low = colors.iter().map(|(c, _)| c[channel]).min().unwrap_or(0);
    let high = colors.iter().map(|(c, _)| c[channel]).max().unwrap_or(0);
    high - low
}

fn average(colors: &[([u8; 3], u64)]) -> [u8; 3] {
    let total: u64 = colors.iter().map(|(_, n)| n).sum::<u64>().max(1);
    let mut sums = [0u64; 3];
    for (color, count) in colors {
        for c in 0..3 {
            sums[c] += color[c] as u64 * count;
        }
    }
    [(sums[0] / total) as u8, (sums[1] / total) as u8, (sums[2] / total) as u8]
}

pub fn shared_palette(crops: &[Rgba], max_colors: usize) -> (Vec<[u8; 3]>, bool) {
    let counts = opaque_colors(crops);
    let mut palette = vec![TRANSPARENT_SLOT];
    if counts.len() < max_colors {
        palette.extend(counts.keys().copied());
        return (palette, true);
    }
    palette.extend(median_cut(counts.into_iter().collect(), max_colors - 1));
    (palette, false)
}

fn nearest(palette: &[[u8; 3]], color: [u8; 3]) -> u8 {
    let distance = |p: &[u8; 3]| (0..3).map(|c| (p[c] as i32 - color[c] as i32).pow(2)).sum::<i32>();
    palette.iter().enumerate().skip(1).min_by_key(|(_, p)| distance(p)).map(|(i, _)| i as u8).unwrap_or(0)
}

pub fn to_indexed(crop: &Rgba, palette: &[[u8; 3]]) -> Vec<u8> {
    let mut cache: HashMap<[u8; 3], u8> = HashMap::new();
    crop.data
        .chunks_exact(4)
        .map(|pixel| {
            if pixel[3] == 0 {
                return 0;
            }
            let color = [pixel[0], pixel[1], pixel[2]];
            *cache.entry(color).or_insert_with(|| nearest(palette, color))
        })
        .collect()
}
