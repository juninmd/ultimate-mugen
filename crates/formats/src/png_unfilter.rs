pub(crate) fn unfilter(ft: u8, line: &mut [u8], prev: &[u8], bpp: usize) -> Result<(), String> {
    let prev = &prev[..line.len()];
    match (ft, bpp) {
        (0, _) => {}
        (2, _) => {
            for (l, &p) in line.iter_mut().zip(prev) {
                *l = l.wrapping_add(p);
            }
        }
        (1, 1) => sub::<1>(line),
        (1, 2) => sub::<2>(line),
        (1, 3) => sub::<3>(line),
        (1, 4) => sub::<4>(line),
        (1, _) => sub_generic(line, bpp),
        (3, 1) => average::<1>(line, prev),
        (3, 2) => average::<2>(line, prev),
        (3, 3) => average::<3>(line, prev),
        (3, 4) => average::<4>(line, prev),
        (3, _) => average_generic(line, prev, bpp),
        (4, 1) => paeth::<1>(line, prev),
        (4, 2) => paeth::<2>(line, prev),
        (4, 3) => paeth::<3>(line, prev),
        (4, 4) => paeth::<4>(line, prev),
        (4, _) => paeth_generic(line, prev, bpp),
        _ => return Err(format!("PNG: bad filter type {ft}")),
    }
    Ok(())
}

fn sub<const N: usize>(line: &mut [u8]) {
    let mut left = [0u8; N];
    for px in line.chunks_exact_mut(N) {
        for i in 0..N {
            px[i] = px[i].wrapping_add(left[i]);
            left[i] = px[i];
        }
    }
}

fn average<const N: usize>(line: &mut [u8], prev: &[u8]) {
    let mut left = [0u8; N];
    for (px, up) in line.chunks_exact_mut(N).zip(prev.chunks_exact(N)) {
        for i in 0..N {
            let avg = ((left[i] as u16 + up[i] as u16) >> 1) as u8;
            px[i] = px[i].wrapping_add(avg);
            left[i] = px[i];
        }
    }
}

#[inline(always)]
fn predictor(a: i16, b: i16, c: i16) -> u8 {
    let pa = (b - c).abs();
    let pb = (a - c).abs();
    let pc = (a + b - c - c).abs();
    if pa <= pb && pa <= pc {
        a as u8
    } else if pb <= pc {
        b as u8
    } else {
        c as u8
    }
}

fn paeth<const N: usize>(line: &mut [u8], prev: &[u8]) {
    let mut left = [0i16; N];
    let mut upper_left = [0i16; N];
    for (px, up) in line.chunks_exact_mut(N).zip(prev.chunks_exact(N)) {
        for i in 0..N {
            let b = up[i] as i16;
            px[i] = px[i].wrapping_add(predictor(left[i], b, upper_left[i]));
            left[i] = px[i] as i16;
            upper_left[i] = b;
        }
    }
}

fn sub_generic(line: &mut [u8], bpp: usize) {
    for x in bpp..line.len() {
        line[x] = line[x].wrapping_add(line[x - bpp]);
    }
}

fn average_generic(line: &mut [u8], prev: &[u8], bpp: usize) {
    for x in 0..line.len() {
        let a = if x >= bpp { line[x - bpp] as u16 } else { 0 };
        line[x] = line[x].wrapping_add(((a + prev[x] as u16) >> 1) as u8);
    }
}

fn paeth_generic(line: &mut [u8], prev: &[u8], bpp: usize) {
    for x in 0..line.len() {
        let a = if x >= bpp { line[x - bpp] as i16 } else { 0 };
        let c = if x >= bpp { prev[x - bpp] as i16 } else { 0 };
        line[x] = line[x].wrapping_add(predictor(a, prev[x] as i16, c));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reference(ft: u8, line: &mut [u8], prev: &[u8], bpp: usize) {
        let len = line.len();
        for x in 0..len {
            let a = if x >= bpp { line[x - bpp] as i16 } else { 0 };
            let b = prev[x] as i16;
            let c = if x >= bpp { prev[x - bpp] as i16 } else { 0 };
            let add = match ft {
                0 => 0,
                1 => a,
                2 => b,
                3 => (a + b) >> 1,
                _ => {
                    let pp = a + b - c;
                    let (pa, pb, pc) = ((pp - a).abs(), (pp - b).abs(), (pp - c).abs());
                    if pa <= pb && pa <= pc { a } else if pb <= pc { b } else { c }
                }
            };
            line[x] = line[x].wrapping_add(add as u8);
        }
    }

    fn noise(seed: &mut u32, len: usize) -> Vec<u8> {
        (0..len)
            .map(|_| {
                *seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                (*seed >> 24) as u8
            })
            .collect()
    }

    #[test]
    fn matches_the_spec_formulas_for_every_filter_and_pixel_size() {
        let mut seed = 7;
        for ft in 0..=4u8 {
            for bpp in 1..=8usize {
                for pixels in [1usize, 2, 5, 33] {
                    let len = pixels * bpp;
                    let prev = noise(&mut seed, len);
                    let raw = noise(&mut seed, len);
                    let (mut want, mut got) = (raw.clone(), raw);
                    reference(ft, &mut want, &prev, bpp);
                    unfilter(ft, &mut got, &prev, bpp).unwrap();
                    assert_eq!(want, got, "filter {ft} bpp {bpp} pixels {pixels}");
                }
            }
        }
    }

    #[test]
    fn rejects_unknown_filter_types() {
        assert!(unfilter(5, &mut [0; 4], &[0; 4], 4).is_err());
    }
}
