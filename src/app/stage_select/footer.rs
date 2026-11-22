use super::*;

impl App {
    pub(super) fn trim_far_previews(&self, view: &[usize]) {
        if TRIMMED_ROW.with(|t| t.replace(self.stage_row)) == self.stage_row {
            return;
        }
        let lo = self.stage_row.saturating_sub(PREVIEW_KEEP_ROWS) * COLS;
        let hi = ((self.stage_row + VIEW_ROWS + PREVIEW_KEEP_ROWS) * COLS).min(view.len());
        let mut keep: Vec<usize> = view[lo..hi]
            .iter()
            .filter_map(|v| v.checked_sub(1))
            .collect();
        keep.sort_unstable();
        self.reg
            .trim_stage_previews(|i| keep.binary_search(&i).is_ok());
    }

    pub(super) fn stage_footer(&self, pos: usize, n: usize, total_rows: usize) {
        let pages = total_rows.div_ceil(VIEW_ROWS).max(1);
        let page = ((pos / COLS) / VIEW_ROWS).min(pages - 1) + 1;
        let tr = crate::i18n::tr3;
        let text = format!(
            "{} {}/{} · {} {page}/{pages} · [Tab] {} · [/] {} · PgUp/PgDn · Home/End",
            tr("Cenário", "Stage", "Escenario"),
            pos + 1,
            n,
            tr("Página", "Page", "Página"),
            tr("filtro", "filter", "filtro"),
            tr("buscar", "search", "buscar"),
        );
        self.ui
            .text(&text, 156.0, 700.0, 15.0, hex("#a79fc4"), Align::Left, false);
        if n == 1 {
            let msg = tr(
                "Nenhum cenário encontrado",
                "No stages found",
                "Ningún escenario encontrado",
            );
            self.ui.text(
                msg,
                40.0 + TILE_W + 13.0 + 10.0,
                TOP + TILE_H / 2.0,
                20.0,
                hex("#ff4d6d"),
                Align::Left,
                true,
            );
        }
    }
}
