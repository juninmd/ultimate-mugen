use super::recolor::Recolor;
use std::sync::Arc;
use ume_engine::chardef::ImageData;

const MARGIN: usize = 1;

pub(super) struct Cropped {
    pub image: Arc<ImageData>,
    pub left: f32,
    pub top: f32,
}

pub(super) fn crop(width: u32, height: u32, rgba: Arc<Vec<u8>>) -> Cropped {
    let bounds = content_box(width as usize, height as usize, &rgba);
    crop_bounded(width, height, rgba, bounds)
}

fn crop_bounded(width: u32, height: u32, rgba: Arc<Vec<u8>>, bounds: Option<[usize; 4]>) -> Cropped {
    let (w, h) = (width as usize, height as usize);
    let Some([x0, y0, x1, y1]) = bounds else {
        return Cropped { image: ImageData::new(width, height, rgba), left: 0.0, top: 0.0 };
    };
    let (x0, y0) = (x0.saturating_sub(MARGIN), y0.saturating_sub(MARGIN));
    let (x1, y1) = ((x1 + MARGIN).min(w), (y1 + MARGIN).min(h));
    if (x0, y0, x1, y1) == (0, 0, w, h) {
        return Cropped { image: ImageData::new(width, height, rgba), left: 0.0, top: 0.0 };
    }
    let mut out = Vec::with_capacity((x1 - x0) * (y1 - y0) * 4);
    for row in y0..y1 {
        out.extend_from_slice(&rgba[(row * w + x0) * 4..(row * w + x1) * 4]);
    }
    Cropped { image: ImageData::new((x1 - x0) as u32, (y1 - y0) as u32, out), left: x0 as f32, top: y0 as f32 }
}

pub(super) fn crop_art(width: u32, height: u32, rgba: Arc<Vec<u8>>, bounds: Option<[usize; 4]>, recolor: Option<&Recolor>, hair_region: Option<[u32; 4]>) -> Cropped {
    match recolor {
        Some(recolor) => crop_recolored(width, height, &rgba, bounds, recolor, hair_region),
        None => crop_bounded(width, height, rgba, bounds),
    }
}

fn crop_recolored(width: u32, height: u32, rgba: &[u8], bounds: Option<[usize; 4]>, recolor: &Recolor, hair_region: Option<[u32; 4]>) -> Cropped {
    let (w, h) = (width as usize, height as usize);
    let [x0, y0, x1, y1] = padded_box(w, h, bounds).unwrap_or([0, 0, w, h]);
    let mut out = Vec::with_capacity((x1 - x0) * (y1 - y0) * 4);
    for row in y0..y1 {
        out.extend_from_slice(&rgba[(row * w + x0) * 4..(row * w + x1) * 4]);
    }
    let shift = |value: u32, by: usize| value.saturating_sub(by as u32);
    let region = hair_region.map(|[rx1, ry1, rx2, ry2]| [shift(rx1, x0), shift(ry1, y0), shift(rx2, x0), shift(ry2, y0)]);
    recolor.apply_in_region(&mut out, (x1 - x0) as u32, region);
    Cropped { image: ImageData::new((x1 - x0) as u32, (y1 - y0) as u32, out), left: x0 as f32, top: y0 as f32 }
}

fn padded_box(w: usize, h: usize, bounds: Option<[usize; 4]>) -> Option<[usize; 4]> {
    let [x0, y0, x1, y1] = bounds?;
    Some([x0.saturating_sub(MARGIN), y0.saturating_sub(MARGIN), (x1 + MARGIN).min(w), (y1 + MARGIN).min(h)])
}

pub(super) fn content_box(w: usize, h: usize, rgba: &[u8]) -> Option<[usize; 4]> {
    if rgba.len() < w * h * 4 {
        return None;
    }
    let rows = rgba.chunks_exact(w * 4);
    let (mut x0, mut x1, mut y0, mut y1) = (w, 0, h, 0);
    for (y, row) in rows.enumerate().take(h) {
        let Some(first) = row.chunks_exact(4).position(|p| p[3] != 0) else { continue };
        let last = row.chunks_exact(4).rposition(|p| p[3] != 0).unwrap_or(first);
        x0 = x0.min(first);
        x1 = x1.max(last + 1);
        y0 = y0.min(y);
        y1 = y + 1;
    }
    (x0 < x1).then_some([x0, y0, x1, y1])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn canvas(w: usize, h: usize, solid: &[(usize, usize)]) -> Arc<Vec<u8>> {
        let mut v = vec![0u8; w * h * 4];
        for &(x, y) in solid {
            v[(y * w + x) * 4..(y * w + x) * 4 + 4].copy_from_slice(&[9, 8, 7, 255]);
        }
        Arc::new(v)
    }

    #[test]
    fn crops_to_content_plus_margin_and_reports_offset() {
        let c = crop(20, 20, canvas(20, 20, &[(5, 6), (8, 9)]));
        assert_eq!((c.image.w, c.image.h, c.left, c.top), (6, 6, 4.0, 5.0));
        assert_eq!(&c.image.rgba[(1 * 6 + 1) * 4..(1 * 6 + 1) * 4 + 4], &[9, 8, 7, 255]);
        assert_eq!(&c.image.rgba[(4 * 6 + 4) * 4..(4 * 6 + 4) * 4 + 4], &[9, 8, 7, 255]);
    }

    #[test]
    fn keeps_edge_touching_and_blank_images_whole() {
        let edge = crop(4, 4, canvas(4, 4, &[(0, 0), (3, 3)]));
        assert_eq!((edge.image.w, edge.image.h, edge.left, edge.top), (4, 4, 0.0, 0.0));
        let blank = crop(4, 4, canvas(4, 4, &[]));
        assert_eq!((blank.image.w, blank.image.h), (4, 4));
    }
}
