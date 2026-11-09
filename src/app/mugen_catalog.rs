use crate::app::App;
use crate::controls::nav;
use crate::gfx::*;
use crate::i18n::{fill, tr3};
use crate::registry::{reason_counts, Quarantined};
use crate::ui::*;

pub(super) const COLS: usize = 4;
pub(super) const ROWS: usize = 3;
pub(super) const TILE_W: f32 = 220.0;
pub(super) const TILE_H: f32 = 136.0;
pub(super) const GAP: f32 = 10.0;
pub(super) const LIST_ROWS: usize = 10;
pub(super) const LIST_H: f32 = 44.0;
pub(super) const TOP: f32 = 218.0;
const GALLERY: u8 = 0;
const QUARANTINE: u8 = 1;
const BUTTON: (f32, f32, f32, f32) = (40.0, 76.0, 330.0, 52.0);

pub(super) fn short(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(max).collect::<String>())
    }
}

impl App {
    pub(crate) fn mugen_catalog(&mut self, view: u8, page: usize) -> Option<(u8, usize)> {
        self.ui.background();
        let loaded: Vec<usize> = (0..self.reg.chars.len()).filter(|&i| self.reg.chars[i].mugen).collect();
        let quarantined: Vec<Quarantined> = self.reg.quarantined.clone();
        let stages = self.reg.stages.iter().filter(|s| s.mugen).count();
        self.catalog_header(loaded.len(), &quarantined, stages);
        self.import_button_row();

        let (mut view, mut page) = (view, page);
        if is_key_pressed(KeyCode::Tab) {
            view = if view == GALLERY { QUARANTINE } else { GALLERY };
            page = 0;
        }
        let per = if view == GALLERY { COLS * ROWS } else { LIST_ROWS };
        let total = if view == GALLERY { loaded.len() } else { quarantined.len() };
        let pages = total.div_ceil(per).max(1);
        if self.c.pressed(nav::RIGHT) && page + 1 < pages {
            page += 1;
        }
        if self.c.pressed(nav::LEFT) && page > 0 {
            page -= 1;
        }
        let page = page.min(pages - 1);

        if view == GALLERY {
            self.catalog_gallery(&loaded, page);
        } else {
            self.catalog_list(&quarantined, page);
        }
        self.catalog_footer(view, page, pages, total);

        if self.c.pressed(nav::BACK) {
            return None;
        }
        Some((view, page))
    }

    fn import_button_row(&mut self) {
        let (x, y, w, h) = BUTTON;
        let hover = self.hover_rect(x, y, w, h);
        let pulse = 0.5 + 0.5 * (self.ui.t * 3.0).sin();
        crate::menu_art::glass(&self.ui, x, y, w, h, 1.0);
        self.ui.rect(x, y, w, h, Color::new(0.13, 0.91, 1.0, if hover { 0.34 } else { 0.18 + 0.06 * pulse }));
        self.ui.rect_lines(x, y, w, h, 2.0, hex("#22e8ff"));
        let label = tr3("IMPORTAR .ZIP", "IMPORT .ZIP", "IMPORTAR .ZIP");
        self.ui.text_bold(label, x + w / 2.0, y + h / 2.0 + 9.0, 26.0, WHITE, hex("#06142a"), Align::Center);
        let hint = if cfg!(target_arch = "wasm32") {
            tr3("Escolha um ou mais .zip de personagens e cenários M.U.G.E.N · ou arraste-os para esta janela", "Pick one or more .zip files of M.U.G.E.N characters and stages · or drag them onto this window", "Elige uno o más .zip de personajes y escenarios M.U.G.E.N · o arrástralos a esta ventana")
        } else {
            tr3("Enter ou clique para escolher .zip · ou coloque pastas em chars/ e stages/", "Press Enter or click to pick .zip files · or place folders in chars/ and stages/", "Pulsa Enter o haz clic para elegir .zip · o coloca carpetas en chars/ y stages/")
        };
        self.ui.text(hint, x + w + 24.0, y + h / 2.0 + 6.0, 16.0, hex("#a49cc0"), Align::Left, false);
        let ui = &self.ui;
        crate::import_button::place(ui.x(x), ui.y(y), w * ui.s, h * ui.s);
        #[cfg(not(target_arch = "wasm32"))]
        {
            if self.c.pressed(nav::OK) || self.clicked_rect(x, y, w, h) {
                self.audio.sfx("confirm");
                crate::native_import::pick();
            }
            if let Some(text) = crate::native_import::take_notice() {
                self.msg = text;
            }
        }
    }

    fn catalog_header(&self, loaded: usize, quarantined: &[Quarantined], stages: usize) {
        self.ui.text(tr3("IMPORTAR M.U.G.E.N", "IMPORT M.U.G.E.N", "IMPORTAR M.U.G.E.N"), 40.0, 56.0, 30.0, hex("#ffcc33"), Align::Left, true);
        let summary = fill(
            tr3("Carregados: {n} personagens   ·   Em quarentena: {q}   ·   Cenários: {s}", "Loaded: {n} characters   ·   Quarantined: {q}   ·   Stages: {s}", "Cargados: {n} personajes   ·   En cuarentena: {q}   ·   Escenarios: {s}"),
            &[("n", &loaded.to_string()), ("q", &quarantined.len().to_string()), ("s", &stages.to_string())],
        );
        self.ui.text(&summary, 40.0, 164.0, 22.0, WHITE, Align::Left, false);
        if quarantined.is_empty() {
            self.ui.text(
                tr3("Nenhum personagem quebrado: todos os arquivos referenciados existem.", "No broken characters: every referenced file exists.", "Ningún personaje roto: todos los archivos referenciados existen."),
                40.0,
                190.0,
                16.0,
                hex("#9fe8a8"),
                Align::Left,
                false,
            );
        }
        for (k, (label, n)) in reason_counts(quarantined).iter().take(2).enumerate() {
            let text = format!("· {}: {n}", super::mugen_gallery::reason_text(label));
            self.ui.text(&text, 40.0, 188.0 + k as f32 * 20.0, 16.0, hex("#ff9b9b"), Align::Left, false);
        }
    }

    fn catalog_footer(&self, view: u8, page: usize, pages: usize, total: usize) {
        let (gallery, quarantine) = (tr3("GALERIA", "GALLERY", "GALERÍA"), tr3("QUARENTENA", "QUARANTINE", "CUARENTENA"));
        let tabs = if view == GALLERY {
            format!("[{gallery}: {total}]   {}", quarantine.to_lowercase())
        } else {
            format!("{}   [{quarantine}: {total}]", gallery.to_lowercase())
        };
        self.ui.text(&tabs, 40.0, 656.0, 18.0, hex("#ffcc33"), Align::Left, true);
        let nav_hint = fill(
            tr3("Página {p}/{n}   ·   TAB alterna   ·   ESQ/DIR páginas   ·   ESC voltar", "Page {p}/{n}   ·   TAB switches   ·   LEFT/RIGHT pages   ·   ESC back", "Página {p}/{n}   ·   TAB alterna   ·   IZQ/DER páginas   ·   ESC volver"),
            &[("p", &(page + 1).to_string()), ("n", &pages.to_string())],
        );
        self.ui.text(&nav_hint, 40.0, 686.0, 15.0, hex("#8a8fa6"), Align::Left, false);
    }
}
