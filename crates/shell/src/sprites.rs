use crate::Gpu;
use std::collections::HashMap;
use ume_render::Rect;

#[derive(Default)]
pub struct SpriteCache { slots: HashMap<u64, Option<Rect>> }

impl SpriteCache {
    pub fn get(&mut self, gpu: &mut Gpu, id: u64, w: u32, h: u32, rgba: &[u8]) -> Option<Rect> {
        *self.slots.entry(id).or_insert_with(|| {
            let r = gpu.rgba_atlas.alloc(w, h)?;
            gpu.renderer.upload_rgba(&gpu.queue, r, rgba);
            Some(r)
        })
    }

    pub fn len(&self) -> usize { self.slots.len() }

    pub fn is_empty(&self) -> bool { self.slots.is_empty() }

    pub fn clear(&mut self, gpu: &mut Gpu) {
        self.slots.clear();
        gpu.rgba_atlas.clear();
    }
}
