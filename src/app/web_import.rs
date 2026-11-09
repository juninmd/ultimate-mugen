use super::*;
use crate::i18n::tr;
use ume_formats::vfs::{content_hash, ZipJob};

const SLICE_BYTES: u64 = 6 << 20;
const STALL_SECS: f32 = 25.0;

/// Web-only incremental import. Each frame inflates a bounded slice of one pack (real progress,
/// never a frozen frame), registers it in a frame of its own, and a stalled loader or a bad pack
/// ends in a message instead of an endless overlay.
pub(crate) struct WasmImport {
    job: Option<ZipJob>,
    pack: String,
    ready: Option<(String, ume_formats::vfs::Vfs)>,
    packs_done: usize,
    stalled: f32,
    problems: usize,
    registering: bool,
}

impl WasmImport {
    pub(super) fn new() -> Self {
        WasmImport { job: None, pack: String::new(), ready: None, packs_done: 0, stalled: 0.0, problems: 0, registering: false }
    }
}

impl App {
    pub(super) fn wasm_import_step(&mut self, dt: f32) {
        if is_key_pressed(KeyCode::Escape) {
            crate::webimport::cancel();
            self.wasm_import = None;
            self.msg = tr("Importação cancelada", "Import cancelled").into();
            return;
        }
        let state = self.wasm_import.as_mut().unwrap();
        if let Some((pack, vfs)) = state.ready.take() {
            state.registering = false;
            self.finish_import(Ok((pack, vfs)));
            self.wasm_import.as_mut().unwrap().packs_done += 1;
            return;
        }
        if let Some(job) = state.job.as_mut() {
            state.stalled = 0.0;
            match job.step(SLICE_BYTES) {
                Ok(false) => {}
                Ok(true) => {
                    let vfs = state.job.take().unwrap().into_vfs();
                    state.ready = Some((std::mem::take(&mut state.pack), vfs));
                    state.registering = true;
                }
                Err(e) => {
                    state.job = None;
                    state.problems += 1;
                    self.msg = format!("{}: {e}", tr("Zip inválido", "Invalid zip"));
                }
            }
            return;
        }
        if let Some(zip) = crate::webimport::take_import() {
            state.stalled = 0.0;
            state.pack = content_hash(&zip);
            match ZipJob::new(zip) {
                Ok(job) => state.job = Some(job),
                Err(e) => {
                    state.problems += 1;
                    self.msg = format!("{}: {e}", tr("Zip inválido", "Invalid zip"));
                }
            }
            return;
        }
        if !crate::webimport::busy() {
            let problems = state.problems;
            self.wasm_import = None;
            if problems > 0 && self.msg.is_empty() {
                self.msg = tr("Importação terminou com erros", "Import finished with errors").into();
            }
            return;
        }
        state.stalled += dt;
        if state.stalled > STALL_SECS {
            crate::webimport::abandon();
            self.wasm_import = None;
            self.msg = tr(
                "O navegador não entregou o arquivo a tempo. Tente importar de novo.",
                "The browser did not hand over the file in time. Try importing again.",
            )
            .into();
        }
    }

    pub(super) fn draw_import_overlay(&mut self) {
        let Some(state) = self.wasm_import.as_ref() else { return };
        let queued = crate::webimport::queued_imports();
        let (ratio, detail) = match (&state.job, state.registering) {
            (Some(job), _) => (
                job.fraction(),
                format!("{} ({}/{})", job.current(), job.entries_done(), job.entries_total()),
            ),
            (None, true) => (1.0, tr("registrando personagens e cenários...", "registering fighters and stages...").to_string()),
            _ => (0.0, tr("lendo o arquivo...", "reading the file...").to_string()),
        };
        let total_packs = state.packs_done + queued + usize::from(state.job.is_some() || state.ready.is_some());
        let label = format!("{} {}/{}", tr("pacote", "pack"), (state.packs_done + 1).min(total_packs.max(1)), total_packs.max(1));
        clear_background(hex("#05030c"));
        self.ui.rect(0.0, 0.0, VW, VH, Color::new(0.015, 0.01, 0.04, 0.9));
        self.ui.text_bold(
            tr("IMPORTANDO RECURSOS MUGEN", "IMPORTING MUGEN RESOURCES"),
            VW / 2.0,
            VH / 2.0 - 56.0,
            36.0,
            WHITE,
            BLACK,
            Align::Center,
        );
        self.ui.text(&label, VW / 2.0, VH / 2.0 - 14.0, 22.0, hex("#d8d0f0"), Align::Center, false);
        let (w, x, y) = (460.0, VW / 2.0 - 230.0, VH / 2.0 + 8.0);
        self.ui.rect(x, y, w, 14.0, Color::new(1.0, 1.0, 1.0, 0.12));
        if state.job.is_some() || state.registering {
            self.ui.rect(x, y, w * ratio.clamp(0.0, 1.0), 14.0, hex("#22e8ff"));
        } else {
            let phase = (self.ui.t * 0.55).fract();
            self.ui.rect(x + (w - 140.0) * phase, y, 140.0, 14.0, hex("#22e8ff"));
        }
        self.ui.rect(x, y, w, 2.0, Color::new(0.13, 0.91, 1.0, 0.65));
        self.ui.text(&detail, VW / 2.0, VH / 2.0 + 44.0, 16.0, hex("#a49cc0"), Align::Center, false);
        self.ui.text(
            tr("ESC cancela · o jogo volta sozinho ao terminar", "ESC cancels · the game returns when done"),
            VW / 2.0,
            VH / 2.0 + 72.0,
            16.0,
            hex("#8a82a8"),
            Align::Center,
            false,
        );
    }
}
