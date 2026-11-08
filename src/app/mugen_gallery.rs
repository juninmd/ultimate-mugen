use super::mugen_catalog::{short, COLS, LIST_H, LIST_ROWS, ROWS, TILE_H, TILE_W, GAP, TOP};
use crate::app::App;
use crate::gfx::*;
use crate::i18n::tr3;
use crate::registry::Quarantined;
use crate::ui::*;

pub(super) fn reason_text(label: &str) -> &'static str {
    match label {
        "arquivo de sprites (.sff) ausente" => tr3("arquivo de sprites (.sff) ausente", "sprite file (.sff) missing", "falta el archivo de sprites (.sff)"),
        "arquivo de animação (.air) ausente" => tr3("arquivo de animação (.air) ausente", "animation file (.air) missing", "falta el archivo de animación (.air)"),
        "arquivo de estados (.cns) ausente" => tr3("arquivo de estados (.cns) ausente", "state file (.cns) missing", "falta el archivo de estados (.cns)"),
        "[Files] sem a chave obrigatória" => tr3("[Files] sem a chave obrigatória", "[Files] is missing a required key", "[Files] sin la clave obligatoria"),
        ".def sem seção [Files]" => tr3(".def sem seção [Files]", ".def has no [Files] section", ".def sin sección [Files]"),
        _ => tr3("arquivo .def não encontrado", ".def file not found", "archivo .def no encontrado"),
    }
}

impl App {
    pub(super) fn catalog_gallery(&mut self, loaded: &[usize], page: usize) {
        let x0 = 40.0;
        let mut streamed = 0;
        for (k, &i) in loaded.iter().skip(page * COLS * ROWS).take(COLS * ROWS).enumerate() {
            let (cx, cy) = (
                x0 + (k % COLS) as f32 * (TILE_W + GAP),
                TOP + (k / COLS) as f32 * (TILE_H + GAP),
            );
            let c = &self.reg.chars[i];
            if !c.has_portrait_attempt() && streamed < 3 {
                c.stream_portrait();
                streamed += 1;
            }
            self.ui.rect(cx, cy, TILE_W, TILE_H, Color::new(0.06, 0.04, 0.12, 0.92));
            self.ui.rect(cx, cy, TILE_W, 4.0, alpha(rgb(c.color), 0.9));
            if let Some(im) = c.portrait().and_then(|p| p.image.clone().map(|im| (im, p.width, p.height))) {
                let (im, pw, ph) = im;
                let tex = self.g.texture(&im, true);
                let k = ((TILE_W - 12.0) / pw).min((TILE_H - 64.0) / ph);
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
            self.ui.text(&short(&c.name, 24), cx + 8.0, cy + TILE_H - 32.0, 16.0, WHITE, Align::Left, true);
            let author = if c.author.is_empty() { tr3("autor desconhecido", "unknown author", "autor desconocido").to_string() } else { format!("{} {}", tr3("por", "by", "por"), c.author) };
            self.ui.text(&short(&author, 30), cx + 8.0, cy + TILE_H - 12.0, 13.0, hex("#8a8fa6"), Align::Left, false);
        }
    }

    pub(super) fn catalog_list(&self, quarantined: &[Quarantined], page: usize) {
        for (k, q) in quarantined.iter().skip(page * LIST_ROWS).take(LIST_ROWS).enumerate() {
            let y = TOP + k as f32 * LIST_H;
            self.ui.rect(40.0, y, 1200.0, LIST_H - 6.0, Color::new(0.06, 0.04, 0.12, 0.92));
            self.ui.text(&short(&q.name, 40), 52.0, y + 6.0, 19.0, WHITE, Align::Left, true);
            let reason = q.problems.first().map(|p| reason_text(crate::registry::reason_label(p))).unwrap_or("");
            self.ui.text(&format!("{reason}  ·  {}", short(&q.def, 70)), 52.0, y + 25.0, 15.0, hex("#ff9b9b"), Align::Left, false);
        }
        if quarantined.is_empty() {
            self.ui.text(tr3("Nada em quarentena.", "Nothing quarantined.", "Nada en cuarentena."), 52.0, TOP + 8.0, 20.0, hex("#9fe8a8"), Align::Left, false);
        }
    }
}
