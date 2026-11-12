use std::collections::BTreeMap;
use tiny_skia::Pixmap;

const OPAQUE_ALPHA: u8 = 40;

pub fn apply(source: &Pixmap) -> Option<Pixmap> {
    let mut counts = BTreeMap::<[u8; 3], u32>::new();
    for pixel in source.data().chunks_exact(4) {
        if pixel[3] >= OPAQUE_ALPHA {
            let alpha = pixel[3] as u32;
            let color = [
                ((pixel[0] as u32 * 255 + alpha / 2) / alpha).min(255) as u8,
                ((pixel[1] as u32 * 255 + alpha / 2) / alpha).min(255) as u8,
                ((pixel[2] as u32 * 255 + alpha / 2) / alpha).min(255) as u8,
            ];
            *counts.entry(color).or_default() += 1;
        }
    }
    let colors: Vec<_> = counts.into_iter().collect();
    let palette = palette(&colors);
    if palette.is_empty() {
        return None;
    }
    let mut result = Pixmap::new(source.width(), source.height())?;
    for (input, output) in source
        .data()
        .chunks_exact(4)
        .zip(result.data_mut().chunks_exact_mut(4))
    {
        if input[3] < OPAQUE_ALPHA {
            output.fill(0);
            continue;
        }
        let alpha = input[3] as u32;
        let color = [
            ((input[0] as u32 * 255 + alpha / 2) / alpha).min(255) as u8,
            ((input[1] as u32 * 255 + alpha / 2) / alpha).min(255) as u8,
            ((input[2] as u32 * 255 + alpha / 2) / alpha).min(255) as u8,
        ];
        let nearest = palette
            .iter()
            .min_by_key(|candidate| distance(color, **candidate))?;
        output[..3].copy_from_slice(nearest);
        output[3] = 255;
    }
    Some(result)
}

fn palette(colors: &[([u8; 3], u32)]) -> Vec<[u8; 3]> {
    let mut buckets = vec![colors.to_vec()];
    while buckets.len() < 47 {
        let Some((index, channel)) = buckets
            .iter()
            .enumerate()
            .filter(|(_, bucket)| bucket.len() > 1)
            .map(|(index, bucket)| (index, range(bucket)))
            .max_by_key(|(_, (_, spread))| *spread)
            .map(|(index, (channel, _))| (index, channel))
        else {
            break;
        };
        let mut bucket = buckets.remove(index);
        bucket.sort_by_key(|(color, _)| color[channel]);
        let total: u64 = bucket.iter().map(|(_, count)| *count as u64).sum();
        let mut subtotal = 0u64;
        let split = bucket
            .iter()
            .position(|(_, count)| {
                subtotal += *count as u64;
                subtotal * 2 >= total
            })
            .unwrap_or(bucket.len() / 2)
            .clamp(1, bucket.len() - 1);
        let right = bucket.split_off(split);
        buckets.push(bucket);
        buckets.push(right);
    }
    buckets
        .iter()
        .map(|bucket| {
            let mut total = 0u64;
            let mut sum = [0u64; 3];
            for (color, count) in bucket {
                total += *count as u64;
                for channel in 0..3 {
                    sum[channel] += color[channel] as u64 * *count as u64;
                }
            }
            [
                (sum[0] / total) as u8,
                (sum[1] / total) as u8,
                (sum[2] / total) as u8,
            ]
        })
        .collect()
}

fn range(bucket: &[([u8; 3], u32)]) -> (usize, u8) {
    let mut result = (0, 0);
    for channel in 0..3 {
        let min = bucket
            .iter()
            .map(|(color, _)| color[channel])
            .min()
            .unwrap_or(0);
        let max = bucket
            .iter()
            .map(|(color, _)| color[channel])
            .max()
            .unwrap_or(0);
        if max - min > result.1 {
            result = (channel, max - min);
        }
    }
    result
}

fn distance(left: [u8; 3], right: [u8; 3]) -> u32 {
    (0..3)
        .map(|channel| left[channel].abs_diff(right[channel]) as u32)
        .map(|value| value * value)
        .sum()
}
