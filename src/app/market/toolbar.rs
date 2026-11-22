use super::badges::format_color;
use super::*;
use ume_market::view::Format;

const BAR_Y: f32 = 46.0;
const BAR_H: f32 = 36.0;

impl App {
    fn market_chip(&self, x: f32, y: f32, w: f32, label: &str, on: bool, tint: Color) -> bool {
        let hover = self.hover_rect(x, y, w, BAR_H);
        self.ui.adapted_plate(x, y, w, BAR_H, (on || hover).then(|| alpha(tint, if on { 0.65 } else { 0.35 })));
        self.ui.text(label, x + w / 2.0, y + 24.0, 15.0, WHITE, Align::Center, on);
        self.clicked_rect(x, y, w, BAR_H)
    }

    pub(super) fn market_toolbar(&self, rx: f32, ry: f32, rw: f32, modal: bool, acts: &mut Vec<Act>) {
        let y = ry + BAR_Y;
        let mut x = rx + 12.0;
        let formats: [(Option<Format>, &str, Color); 3] = [
            (None, mt("Todos"), hex("#22e8ff")),
            (Some(Format::Mugen), Format::Mugen.label(), format_color(Format::Mugen)),
            (Some(Format::Umf), Format::Umf.label(), format_color(Format::Umf)),
        ];
        for (f, label, tint) in formats {
            if self.market_chip(x, y, 84.0, label, self.market_ui.format == f, tint) && !modal {
                acts.push(Act::SetFormat(f));
            }
            x += 88.0;
        }
        x += 8.0;
        let label = format!("{} · {}", mt("Desatualizados"), self.market_ui.outdated_total);
        if self.market_chip(x, y, 178.0, &label, self.market_ui.outdated_only, hex("#ff9e3d")) && !modal {
            acts.push(Act::ToggleOutdated);
        }
        x += 186.0;
        if self.market_chip(x, y, 150.0, mt("Não instalados"), self.market_ui.missing_only, hex("#2ecc71")) && !modal {
            acts.push(Act::ToggleMissing);
        }
        let grid = self.market.state.grid_view;
        let (w, gx) = (76.0, rx + rw - 12.0 - 152.0);
        for (i, (label, is_grid)) in [(mt("Lista"), false), (mt("Grade"), true)].into_iter().enumerate() {
            if self.market_chip(gx + i as f32 * (w + 0.0), y, w, label, grid == is_grid, hex("#b388ff")) && !modal && grid != is_grid {
                acts.push(Act::ToggleLayout);
            }
        }
    }
}
