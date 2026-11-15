use super::*;
use crate::char_tuning;

impl CharEntry {
    pub(super) fn tuned(&self, r: Result<CharDef, String>) -> Result<CharDef, String> {
        r.map(|mut d| {
            char_tuning::get(&self.key).apply(&mut d);
            d
        })
    }

    pub fn drop_loaded(&self) {
        self.cache.borrow_mut().clear();
        self.errs.borrow_mut().clear();
    }
}
