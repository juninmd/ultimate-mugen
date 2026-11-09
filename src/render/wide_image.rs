use super::*;
use std::cell::RefCell;

const CHUNK_WIDTH: u32 = 2048;

thread_local! {
    static CHUNKS: RefCell<HashMap<u64, Arc<Vec<(u32, Arc<ImageData>)>>>> = RefCell::new(HashMap::new());
}

pub(crate) fn split_columns(image: &ImageData, chunk_width: u32) -> Vec<(u32, Arc<ImageData>)> {
    let row_bytes = image.w as usize * 4;
    let mut chunks = Vec::new();
    let mut x0 = 0;
    while x0 < image.w {
        let width = chunk_width.min(image.w - x0);
        let mut rgba = Vec::with_capacity(width as usize * image.h as usize * 4);
        for row in 0..image.h as usize {
            let start = row * row_bytes + x0 as usize * 4;
            rgba.extend_from_slice(&image.rgba[start..start + width as usize * 4]);
        }
        chunks.push((x0, ImageData::new(width, image.h, rgba)));
        x0 += width;
    }
    chunks
}

fn chunks_of(image: &ImageData) -> Arc<Vec<(u32, Arc<ImageData>)>> {
    CHUNKS.with(|cache| {
        cache
            .borrow_mut()
            .entry(image.id)
            .or_insert_with(|| Arc::new(split_columns(image, CHUNK_WIDTH)))
            .clone()
    })
}

impl Gfx {
    pub(super) fn draw_wide_image(
        &mut self,
        image: &Arc<ImageData>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    ) {
        if image.w <= CHUNK_WIDTH {
            let texture = self.texture(image, true);
            draw_texture_ex(
                &texture,
                x,
                y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(w, h)),
                    ..Default::default()
                },
            );
            return;
        }
        let scale = w / image.w as f32;
        let chunks = chunks_of(image);
        for (x0, chunk) in chunks.iter() {
            let left = x + *x0 as f32 * scale;
            let right = x + (*x0 + chunk.w) as f32 * scale;
            let texture = self.texture(chunk, true);
            draw_texture_ex(
                &texture,
                left,
                y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(right - left, h)),
                    ..Default::default()
                },
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn columns_reassemble_to_the_source_and_fit_the_limit() {
        let (w, h) = (4180u32, 3u32);
        let rgba: Vec<u8> = (0..w * h * 4).map(|i| (i % 251) as u8).collect();
        let image = ImageData::new(w, h, rgba.clone());
        let chunks = split_columns(&image, CHUNK_WIDTH);
        assert_eq!(chunks.len(), 3);
        assert!(chunks.iter().all(|(_, c)| c.w <= CHUNK_WIDTH && c.h == h));
        let mut rebuilt = vec![0u8; rgba.len()];
        for (x0, c) in &chunks {
            for row in 0..h as usize {
                let dst = (row * w as usize + *x0 as usize) * 4;
                let src = row * c.w as usize * 4;
                rebuilt[dst..dst + c.w as usize * 4]
                    .copy_from_slice(&c.rgba[src..src + c.w as usize * 4]);
            }
        }
        assert_eq!(rebuilt, rgba);
    }
}
