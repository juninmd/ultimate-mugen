use super::mugen_catalog::{short, LIST_H, LIST_ROWS, TOP};
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
