use super::regions::Mask;

pub type Box4 = (i32, i32, i32, i32);

fn bounds(mask: &Mask) -> Option<(i32, i32, i32, i32)> {
    let mut found: Option<(i32, i32, i32, i32)> = None;
    for y in 0..mask.height {
        for x in 0..mask.width {
            if mask.at(x, y) {
                let (x, y) = (x as i32, y as i32);
                let b = found.get_or_insert((x, x, y, y));
                *b = (b.0.min(x), b.1.max(x), b.2.min(y), b.3.max(y));
            }
        }
    }
    found
}

pub fn anatomical_boxes(mask: &Mask, axis: (i32, i32), zones: usize, margin: i32) -> Vec<Box4> {
    let Some((min_x, max_x, min_y, max_y)) = bounds(mask) else { return Vec::new() };
    let (ax, ay) = axis;
    let height = max_y - min_y;
    if height < 30 || zones <= 1 {
        return vec![(min_x - ax + margin, min_y - ay + margin, max_x - ax - margin, max_y - ay)];
    }
    let splits: Vec<i32> = if height < 50 || zones == 2 {
        vec![min_y, min_y + (height as f64 * 0.48) as i32, max_y + 1]
    } else {
        vec![min_y, min_y + (height as f64 * 0.22) as i32, min_y + (height as f64 * 0.62) as i32, max_y + 1]
    };
    let mut boxes = Vec::new();
    for pair in splits.windows(2) {
        let (y0, y1) = (pair[0], pair[1]);
        let counts: Vec<i32> = (0..mask.width).map(|x| (y0..y1).filter(|&y| mask.at(x, y as usize)).count() as i32).collect();
        if counts.iter().all(|&c| c == 0) {
            continue;
        }
        let needed = ((y1 - y0) as f64 * 0.08).max(1.0);
        let mut columns: Vec<usize> = (0..mask.width).filter(|&x| counts[x] as f64 >= needed).collect();
        if columns.is_empty() {
            columns = (0..mask.width).filter(|&x| counts[x] > 0).collect();
        }
        let mut x0 = columns[0] as i32 - ax + margin;
        let mut x1 = columns[columns.len() - 1] as i32 - ax - margin;
        if x1 - x0 < 8 {
            let mid = (x0 + x1).div_euclid(2);
            x0 = mid - 4;
            x1 = mid + 4;
        }
        let mut bottom = y1 - 1 - ay;
        if y1 >= max_y {
            bottom = bottom.max(0);
        }
        boxes.push((x0, y0 - ay + margin, x1, bottom));
    }
    boxes
}

pub fn attack_boxes(mask: &Mask, axis: (i32, i32), reach_margin: i32) -> Vec<Box4> {
    let (ax, ay) = axis;
    let front: Vec<(i32, i32)> = (0..mask.height)
        .flat_map(|y| (0..mask.width).map(move |x| (x, y)))
        .filter(|&(x, y)| mask.at(x, y) && x as i32 >= ax)
        .map(|(x, y)| (x as i32, y as i32))
        .collect();
    let Some(reach) = front.iter().map(|p| p.0).max() else { return Vec::new() };
    let distance = reach - ax;
    if distance < 12 {
        return Vec::new();
    }
    let cutoff = reach - 12.max((distance as f64 * 0.35) as i32);
    let tip: Vec<&(i32, i32)> = front.iter().filter(|p| p.0 >= cutoff).collect();
    let min_x = tip.iter().map(|p| p.0).min().unwrap_or(0);
    let max_x = tip.iter().map(|p| p.0).max().unwrap_or(0);
    let min_y = tip.iter().map(|p| p.1).min().unwrap_or(0);
    let max_y = tip.iter().map(|p| p.1).max().unwrap_or(0);
    vec![(min_x - ax - reach_margin, min_y - ay - reach_margin, max_x - ax + reach_margin, max_y - ay + reach_margin)]
}

pub fn interpolate_boxes(first: &[Box4], second: &[Box4], steps: usize) -> Result<Vec<Vec<Box4>>, String> {
    if first.len() != second.len() {
        return Err("box lists must have the same length".into());
    }
    let mix = |a: i32, b: i32, t: f64| (a as f64 + (b - a) as f64 * t).round() as i32;
    Ok((1..=steps)
        .map(|step| {
            let t = step as f64 / (steps + 1) as f64;
            first.iter().zip(second).map(|(a, b)| (mix(a.0, b.0, t), mix(a.1, b.1, t), mix(a.2, b.2, t), mix(a.3, b.3, t))).collect()
        })
        .collect())
}
