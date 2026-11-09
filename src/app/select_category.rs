use super::*;

pub(crate) const CATEGORY_COUNT: u8 = 4;

impl App {
    pub(crate) fn category_matches(&self, i: usize) -> bool {
        let e = &self.reg.chars[i];
        match self.sel_filter {
            1 => !e.mugen && !e.boss,
            2 => e.mugen,
            3 => e.boss,
            _ => true,
        }
    }

    pub(crate) fn category_label(&self) -> &'static str {
        [
            crate::i18n::t("select.filter_all"),
            crate::i18n::t("select.filter_originals"),
            crate::i18n::t("select.filter_mugen"),
            crate::i18n::t("select.filter_bosses"),
        ][self.sel_filter as usize % CATEGORY_COUNT as usize]
    }
}
