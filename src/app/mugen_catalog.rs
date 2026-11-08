//! M.U.G.E.N catalogue screen: what loaded, what was quarantined and why, and a gallery of loaded characters.
use crate::app::App;
use crate::controls::nav;
use crate::gfx::*;
use crate::registry::{reason_counts, Quarantined};
use crate::ui::*;

const COLS: usize = 4;
const ROWS: usize = 3;
const TILE_W: f32 = 220.0;
const TILE_H: f32 = 136.0;
const GAP: f32 = 10.0;
const LIST_ROWS: usize = 10;
const LIST_H: f32 = 44.0;
const TOP: f32 = 218.0;
const GALLERY: u8 = 0;
const QUARANTINE: u8 = 1;

fn short(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(max).collect::<String>())
    }
}

impl App {
    /// Returns the next `(view, page)`, or `None` when the player leaves the screen.
    pub(crate) fn mugen_catalog(&mut self, view: u8, page: usize) -> Option<(u8, usize)> {
        self.ui.background();
        let loaded: Vec<usize> = (0..self.reg.chars.len())
            .filter(|&i| self.reg.chars[i].mugen)
            .collect();
        let quarantined: Vec<Quarantined> = self.reg.quarantined.clone();
        let stages = self.reg.stages.iter().filter(|s| s.mugen).count();
        self.catalog_header(loaded.len(), &quarantined, stages);

        let (mut view, mut page) = (view, page);
        if is_key_pressed(KeyCode::Tab) {
            view = if view == GALLERY { QUARANTINE } else { GALLERY };
            page = 0;
        }
        let per = if view == GALLERY {
            COLS * ROWS
        } else {
            LIST_ROWS
        };
        let total = if view == GALLERY {
            loaded.len()
        } else {
            quarantined.len()
        };
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

        if self.c.pressed(nav::BACK | nav::OK) {
            return None;
        }
        Some((view, page))
    }

    fn catalog_header(&self, loaded: usize, quarantined: &[Quarantined], stages: usize) {
        self.ui.text(
            "PERSONAGENS E CENÁRIOS M.U.G.E.N",
            40.0,
            56.0,
            30.0,
            hex("#ffcc33"),
            Align::Left,
            true,
        );
        self.ui.text(
            &format!(
                "Carregados: {loaded} personagens   ·   Em quarentena: {}   ·   Cenários: {stages}",
                quarantined.len()
            ),
            40.0,
            98.0,
            22.0,
            WHITE,
            Align::Left,
            false,
        );
        if quarantined.is_empty() {
            self.ui.text(
                "Nenhum personagem quebrado: todos os arquivos referenciados existem.",
                40.0,
                130.0,
                18.0,
                hex("#9fe8a8"),
                Align::Left,
                false,
            );
        }
        for (k, (label, n)) in reason_counts(quarantined).iter().take(3).enumerate() {
            self.ui.text(
                &format!("· {label}: {n}"),
                40.0,
                130.0 + k as f32 * 24.0,
                18.0,
                hex("#ff9b9b"),
                Align::Left,
                false,
            );
        }
    }

    fn catalog_gallery(&mut self, loaded: &[usize], page: usize) {
        let x0 = 40.0;
        let mut streamed = 0;
        for (k, &i) in loaded
            .iter()
            .skip(page * COLS * ROWS)
            .take(COLS * ROWS)
            .enumerate()
        {
            let (cx, cy) = (
                x0 + (k % COLS) as f32 * (TILE_W + GAP),
                TOP + (k / COLS) as f32 * (TILE_H + GAP),
            );
            let c = &self.reg.chars[i];
            if !c.has_portrait_attempt() && streamed < 3 {
                c.stream_portrait();
                streamed += 1;
            }
            self.ui
                .rect(cx, cy, TILE_W, TILE_H, Color::new(0.06, 0.04, 0.12, 0.92));
            self.ui.rect(cx, cy, TILE_W, 4.0, alpha(rgb(c.color), 0.9));
            if let Some(im) = c
                .portrait()
                .and_then(|p| p.image.clone().map(|im| (im, p.width, p.height)))
            {
                let (im, pw, ph) = im;
                let tex = self.g.texture(&im, true);
                let k = ((TILE_W - 12.0) / pw).min((TILE_H - 52.0) / ph);
                draw_texture_ex(
                    &tex,
                    self.ui.x(cx + (TILE_W - pw * k) / 2.0),
                    self.ui.y(cy + 6.0),
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(pw * k * self.ui.s, ph * k * self.ui.s)),
                        ..Default::default()
                    },
                );
            }
            self.ui.text(
                &short(&c.name, 24),
                cx + 8.0,
                cy + TILE_H - 40.0,
                16.0,
                WHITE,
                Align::Left,
                true,
            );
            let author = if c.author.is_empty() {
                "autor desconhecido".to_string()
            } else {
                format!("por {}", c.author)
            };
            self.ui.text(
                &short(&author, 30),
                cx + 8.0,
                cy + TILE_H - 20.0,
                13.0,
                hex("#8a8fa6"),
                Align::Left,
                false,
            );
        }
    }

    fn catalog_list(&self, quarantined: &[Quarantined], page: usize) {
        for (k, q) in quarantined
            .iter()
            .skip(page * LIST_ROWS)
            .take(LIST_ROWS)
            .enumerate()
        {
            let y = TOP + k as f32 * LIST_H;
            self.ui.rect(
                40.0,
                y,
                1200.0,
                LIST_H - 6.0,
                Color::new(0.06, 0.04, 0.12, 0.92),
            );
            self.ui.text(
                &short(&q.name, 40),
                52.0,
                y + 6.0,
                19.0,
                WHITE,
                Align::Left,
                true,
            );
            let reason = q
                .problems
                .first()
                .map(|p| crate::registry::reason_label(p))
                .unwrap_or("");
            self.ui.text(
                &format!("{reason}  ·  {}", short(&q.def, 70)),
                52.0,
                y + 25.0,
                15.0,
                hex("#ff9b9b"),
                Align::Left,
                false,
            );
        }
        if quarantined.is_empty() {
            self.ui.text(
                "Nada em quarentena.",
                52.0,
                TOP + 8.0,
                20.0,
                hex("#9fe8a8"),
                Align::Left,
                false,
            );
        }
    }

    fn catalog_footer(&self, view: u8, page: usize, pages: usize, total: usize) {
        let tabs = if view == GALLERY {
            format!("[GALERIA: {total}]   quarentena")
        } else {
            format!("galeria   [QUARENTENA: {total}]")
        };
        self.ui
            .text(&tabs, 40.0, 656.0, 18.0, hex("#ffcc33"), Align::Left, true);
        self.ui.text(
            &format!(
                "Página {}/{}   ·   TAB alterna   ·   ESQ/DIR páginas   ·   ESC voltar",
                page + 1,
                pages
            ),
            40.0,
            686.0,
            15.0,
            hex("#8a8fa6"),
            Align::Left,
            false,
        );
    }
}
