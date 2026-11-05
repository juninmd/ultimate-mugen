use super::{CharEntry, Src};
use std::sync::Arc;
use ume_engine::chardef::CharDef;
use ume_engine::native::{compile_native_preview, sprite_pack_revision};

impl CharEntry {
    pub(super) fn refresh_sprite_pack(&self) {
        let revision = match &self.src {
            Src::Native(character) => sprite_pack_revision(&character.id),
            Src::Mugen { .. } => return,
        };
        if revision != self.sprite_revision.replace(revision) {
            self.unload();
        }
    }
    pub fn load_preview(&self, pal: i32) -> Result<Arc<CharDef>, String> {
        if let Some(d) = self.peek(pal) {
            self.touch();
            return Ok(d);
        }
        match &self.src {
            Src::Native(nc) => {
                let d = Arc::new(compile_native_preview(nc, pal));
                self.preview.borrow_mut().insert(pal, d.clone());
                self.touch();
                Ok(d)
            }
            Src::Mugen { .. } => self.load(pal),
        }
    }
    pub fn peek(&self, pal: i32) -> Option<Arc<CharDef>> {
        self.refresh_sprite_pack();
        if let Some(d) = self.cache.borrow().get(&pal) {
            return Some(d.clone());
        }
        self.preview.borrow().get(&pal).cloned()
    }
}
