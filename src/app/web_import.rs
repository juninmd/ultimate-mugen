use super::*;

/// Web-only incremental import: one archive is parsed per frame, with a painted loading
/// overlay first, so a batch of persisted packs never runs in a single frozen frame.
pub(crate) struct WasmImport {
    pub(super) pending: Option<Vec<u8>>,
    pub(super) show_overlay: bool,
}

impl App {
    /// One incremental step of the web importer: take a pack, paint one loading frame, then parse
    /// it. Parsing the archive itself is still synchronous (wasm has no threads), but no batch of
    /// persisted packs is ever drained inside a single frame.
    pub(super) fn wasm_import_step(&mut self) {
        if self
            .wasm_import
            .as_ref()
            .is_none_or(|s| s.pending.is_none())
        {
            let Some(zip) = crate::webimport::take_import() else {
                if !crate::webimport::busy() {
                    self.wasm_import = None;
                }
                return;
            };
            let state = self.wasm_import.get_or_insert_with(|| WasmImport {
                pending: None,
                show_overlay: false,
            });
            state.pending = Some(zip);
            state.show_overlay = false;
        }
        let state = self.wasm_import.as_mut().unwrap();
        if !state.show_overlay {
            state.show_overlay = true;
            return;
        }
        let zip = state.pending.take().unwrap();
        let r = crate::registry::read_zip(&zip);
        self.finish_import(r);
        let left = crate::webimport::queued_imports();
        let state = self.wasm_import.as_mut().unwrap();
        state.show_overlay = false;
        if left == 0 && !crate::webimport::busy() {
            self.wasm_import = None;
        }
    }

    pub(super) fn draw_import_overlay(&mut self) {
        let Some(state) = self.wasm_import.as_ref() else {
            return;
        };
        let left = crate::webimport::queued_imports() + usize::from(state.pending.is_some());
        let busy = crate::webimport::busy();
        clear_background(hex("#05030c"));
        self.ui
            .rect(0.0, 0.0, VW, VH, Color::new(0.015, 0.01, 0.04, 0.9));
        self.ui.text_bold(
            "IMPORTANDO RECURSOS MUGEN",
            VW / 2.0,
            VH / 2.0 - 56.0,
            36.0,
            WHITE,
            BLACK,
            Align::Center,
        );
        let label = if left > 0 {
            format!("{left} pacote(s) na fila")
        } else if busy {
            "lendo pacotes...".to_string()
        } else {
            "finalizando".to_string()
        };
        self.ui.text(
            &label,
            VW / 2.0,
            VH / 2.0 - 14.0,
            22.0,
            hex("#d8d0f0"),
            Align::Center,
            false,
        );
        let (w, x, y) = (460.0, VW / 2.0 - 230.0, VH / 2.0 + 8.0);
        self.ui.rect(x, y, w, 14.0, Color::new(1.0, 1.0, 1.0, 0.12));
        let phase = (self.ui.t * 0.55).fract();
        self.ui
            .rect(x + (w - 140.0) * phase, y, 140.0, 14.0, hex("#22e8ff"));
        self.ui
            .rect(x, y, w, 2.0, Color::new(0.13, 0.91, 1.0, 0.65));
        self.ui.text(
            "O jogo volta sozinho quando o pacote terminar.",
            VW / 2.0,
            VH / 2.0 + 52.0,
            18.0,
            hex("#a49cc0"),
            Align::Center,
            false,
        );
    }
}
