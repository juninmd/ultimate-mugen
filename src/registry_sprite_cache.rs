use super::{CharEntry, Src};
use std::sync::Arc;
use ume_engine::chardef::CharDef;
use std::cell::RefCell;
use std::collections::HashMap;
use ume_engine::chardef::Sprite;
use ume_engine::native::{compile_native_preview, select_portrait, sprite_pack_revision};

thread_local! {
    static TILE_PORTRAITS: RefCell<HashMap<(String, i32), (u64, Option<Sprite>)>> = RefCell::new(HashMap::new());
}

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
    pub fn tile_portrait(&self, pal: i32) -> Option<Sprite> {
        let Src::Native(character) = &self.src else { return self.portrait() };
        let revision = sprite_pack_revision(&character.id);
        let key = (character.id.clone(), pal);
        if let Some((cached, sprite)) = TILE_PORTRAITS.with(|cache| cache.borrow().get(&key).cloned()) {
            if cached == revision {
                return sprite;
            }
        }
        let sprite = ume_formats::perf::time(&format!("portrait:{}", character.id), || select_portrait(character, pal));
        TILE_PORTRAITS.with(|cache| cache.borrow_mut().insert(key, (revision, sprite.clone())));
        sprite
    }
    pub fn tile_art_ready(&self) -> bool {
        match &self.src {
            Src::Native(character) => {
                !cfg!(target_arch = "wasm32") || ume_engine::native::has_sprite_pack(&character.id)
            }
            Src::Mugen { .. } => true,
        }
    }
    pub fn stats(&self) -> (f64, f64, f64) {
        if let Some(d) = self.peek(1) {
            (
                d.c("data.attack", 100.0),
                d.c("data.life", 1000.0),
                d.c("velocity.walk.fwd.x", 2.4),
            )
        } else {
            match &self.src {
                Src::Native(nc) => (
                    nc.stats.attack.unwrap_or(100.0),
                    nc.stats.life.unwrap_or(1000.0),
                    nc.stats.walk.map(|w| w[0]).unwrap_or(2.4),
                ),
                Src::Mugen { vfs, def } => {
                    ume_engine::loader::load_mugen_stats(vfs, def)
                        .unwrap_or((100.0, 1000.0, 2.4))
                }
            }
        }
    }
    pub fn load_preview(&self, pal: i32) -> Result<Arc<CharDef>, String> {
        if let Some(d) = self.peek(pal) {
            self.touch();
            return Ok(d);
        }
        match &self.src {
            Src::Native(nc) => {
                let d = Arc::new(ume_formats::perf::time(&format!("preview:{}", nc.id), || compile_native_preview(nc, pal)));
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
