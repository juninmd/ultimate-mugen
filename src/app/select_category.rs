use super::*;

pub(crate) const CATEGORY_COUNT: u8 = 6;

impl App {
    pub(crate) fn set_select_filter(&mut self, filter: u8) {
        self.sel_filter = filter % CATEGORY_COUNT;
    }

    pub(crate) fn category_matches(&self, i: usize) -> bool {
        let e = &self.reg.chars[i];
        let kind = match self.sel_filter {
            1 => !e.mugen && !e.boss,
            2 => e.mugen,
            3 => e.boss,
            super::select_marks::FILTER_FAVORITES => self.is_favorite(i),
            super::select_marks::FILTER_RECENT => self.recent_rank(i).is_some(),
            _ => true,
        };
        kind && self.author_matches(i) && self.search_matches(i)
    }

    pub(crate) fn category_label(&self) -> &'static str {
        Self::category_labels()[self.sel_filter as usize % CATEGORY_COUNT as usize]
    }

    pub(crate) fn category_labels() -> [&'static str; CATEGORY_COUNT as usize] {
        [
            crate::i18n::t("select.filter_all"),
            crate::i18n::t("select.filter_originals"),
            crate::i18n::t("select.filter_mugen"),
            crate::i18n::t("select.filter_bosses"),
            crate::i18n::tr3("FAVORITOS", "FAVORITES", "FAVORITOS"),
            crate::i18n::tr3("RECENTES", "RECENT", "RECIENTES"),
        ]
    }
}
