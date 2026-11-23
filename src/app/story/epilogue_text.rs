use crate::app::App;
use crate::gfx::*;
use crate::i18n::tr3;
use crate::poly;
use crate::story::epilogue::{Beat, Speaker};
use crate::ui::*;

const BOX_Y: f32 = 566.0;
const BOX_H: f32 = 88.0;
const TYPE_SPEED: f32 = 48.0;

impl App {
    pub(super) fn epilogue_line(
        &mut self,
        beat: &Beat,
        t: f32,
        accent: Color,
        boss: Option<usize>,
    ) -> bool {
        let (name, color) = match beat.speaker {
            Speaker::Hero => (self.reg.chars[self.sel.0].name.to_uppercase(), accent),
            Speaker::Boss => match boss {
                Some(index) => (
                    self.reg.chars[index].name.to_uppercase(),
                    rgb(self.reg.chars[index].color),
                ),
                None => (String::new(), hex("#a49cc0")),
            },
            Speaker::Narrator => (String::new(), hex("#a49cc0")),
        };
        let s = self.ui.s;
        let panel = |x: f32, y: f32, w: f32, h: f32| -> Vec<Vec2> {
            [(x + 12.0, y), (x + w, y), (x + w - 12.0, y + h), (x, y + h)]
                .iter()
                .map(|&(a, b)| vec2(self.ui.x(a), self.ui.y(b)))
                .collect()
        };
        let frame = panel(90.0, BOX_Y, 1100.0, BOX_H);
        poly::grad_v(
            &frame,
            Color::new(0.05, 0.03, 0.12, 0.9),
            Color::new(0.02, 0.01, 0.06, 0.9),
        );
        poly::outline(&frame, 2.0 * s, alpha(color, 0.9));
        if !name.is_empty() {
            poly::grad_h(
                &panel(90.0, BOX_Y - 30.0, 380.0, 30.0),
                alpha(color, 0.95),
                alpha(color, 0.35),
            );
            self.ui
                .text(&name, 110.0, BOX_Y - 8.0, 20.0, WHITE, Align::Left, true);
        }
        let total = beat.text.chars().count();
        let shown = ((t * TYPE_SPEED) as usize).min(total);
        let italic = beat.speaker == Speaker::Narrator;
        let ink = if italic { hex("#e8e2f8") } else { WHITE };
        let wrapped = match self.epi_wrap.take() {
            Some(cached) if cached.0 == beat.text => cached,
            _ => {
                let lines = crate::profile::wrap(&beat.text, 1040.0, |line| self.ui.measure(line, 22.0, false));
                (beat.text.clone(), lines)
            }
        };
        for (row, line) in crate::story::typewriter::reveal(&wrapped.1, shown).into_iter().enumerate() {
            self.ui.text(
                line,
                116.0,
                BOX_Y + 34.0 + row as f32 * 28.0,
                22.0,
                ink,
                Align::Left,
                false,
            );
        }
        self.epi_wrap = Some(wrapped);
        if shown >= total {
            let blink = 0.5 + 0.5 * (self.ui.t * 5.0).sin().abs();
            let tip = vec2(self.ui.x(1162.0), self.ui.y(BOX_Y + BOX_H - 16.0));
            poly::solid(
                &[
                    tip + vec2(-7.0, -8.0) * s,
                    tip + vec2(8.0, 0.0) * s,
                    tip + vec2(-7.0, 8.0) * s,
                ],
                Color::new(1.0, 1.0, 1.0, blink),
            );
        }
        shown >= total
    }

    pub(super) fn epilogue_end_card(&mut self, title: &str, accent: Color) {
        let hero = self.reg.chars[self.sel.0].name.to_uppercase();
        let end = tr3("FIM", "THE END", "FIN");
        self.ui
            .text_bold(end, 860.0, 300.0, 110.0, WHITE, BLACK, Align::Center);
        self.ui.rect(660.0, 330.0, 400.0, 4.0, accent);
        self.ui
            .text(&hero, 860.0, 384.0, 34.0, accent, Align::Center, true);
        self.ui.text(
            title,
            860.0,
            424.0,
            22.0,
            hex("#e8e2f8"),
            Align::Center,
            true,
        );
    }
}
