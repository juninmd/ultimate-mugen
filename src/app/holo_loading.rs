use super::*;
use crate::holo::{self, CardState};

const CYAN: Color = Color::new(0.35, 0.9, 1.0, 1.0);

pub(super) struct Card {
    tex: Option<(Texture2D, f32, f32)>,
    pub(super) name: String,
    pub(super) side: usize,
    pub(super) state: CardState,
}

impl App {
    fn holo_card_data(&mut self, plan: &[(usize, usize, usize)], step: usize) -> Vec<Card> {
        let mut cards = vec![];
        for (k, &(side, _, r)) in plan.iter().enumerate() {
            let e = &self.reg.chars[r];
            if e.mugen && k <= step + 6 {
                e.ensure_portrait();
            }
            let por = if e.mugen {
                e.portrait()
            } else {
                e.tile_portrait(1)
                    .or_else(|| e.peek(1).and_then(|d| d.sprites.get(&(9000, 1)).cloned()))
            };
            let name = e.name.clone();
            let tex = por.and_then(|p| {
                let (w, h) = (p.width, p.height);
                p.image.clone().map(|im| (self.g.texture(&im, true), w, h))
            });
            cards.push(Card { tex, name, side, state: holo::card_state(k, step) });
        }
        cards
    }

    pub(super) fn holo_card_named(&mut self, name: &str, state: CardState, side: usize) -> Card {
        let found = self.reg.chars.iter().position(|e| e.name == name);
        let mut plan = vec![];
        if let Some(i) = found {
            plan.push((side, 0, i));
        }
        let mut cards = self.holo_card_data(&plan, if state == CardState::Ready { 1 } else { 0 });
        match cards.pop() {
            Some(mut card) => {
                card.state = state;
                card
            }
            None => Card { tex: None, name: name.to_string(), side, state },
        }
    }

    pub(super) fn holo_chips(&mut self, step: u8, cy: f32, w: f32, h: f32) {
        let plan = self.load_plan();
        let cards = self.holo_card_data(&plan, step as usize);
        let gap = 14.0;
        let total = cards.len() as f32 * w + (cards.len().saturating_sub(1)) as f32 * gap;
        let mut x = VW / 2.0 - total / 2.0;
        for c in &cards {
            self.holo_paint(c, x, cy, w, h, false);
            x += w + gap;
        }
    }

    pub(crate) fn holo_loading_screen(&mut self, step: u8, p: f32, label: &str) {
        let plan = self.load_plan();
        self.holo_mosaic_screen(&plan, step as usize, p, label);
    }

    pub(crate) fn holo_mosaic_screen(&mut self, plan: &[(usize, usize, usize)], step: usize, p: f32, label: &str) {
        crate::screen_art::ScreenArt::Loading.background(&self.ui);
        self.ui.logo(VW / 2.0, 150.0, 84.0);
        let shown = holo::revealed(step, plan.len());
        let cards = self.holo_card_data(&plan[..shown], step);
        let (ax, ay, aw, ah) = (60.0, 215.0, VW - 120.0, 340.0);
        let m = holo::mosaic(cards.len(), aw, ah, 190.0);
        for (i, c) in cards.iter().enumerate() {
            let (x, y) = holo::origin(&m, i, cards.len(), ax, ay, aw, ah);
            self.holo_paint(c, x, y, m.w, m.h, m.labels);
        }
        let done = plan.len().min(step);
        let count = format!("{done} / {}", plan.len());
        self.ui.text(&count, VW / 2.0, 574.0, 16.0, hex("#a49cc0"), Align::Center, false);
        self.ui.progress_bar(VW / 2.0 - 260.0, 600.0, 520.0, p, label);
        let tweaks = crate::tweaks::current();
        if tweaks.title_crt {
            crate::title_crt::draw(&self.ui, self.ui.t, !tweaks.reduce_motion);
        }
    }

    pub(super) fn holo_paint(&self, c: &Card, x: f32, y: f32, w: f32, h: f32, labels: bool) {
        let ui = &self.ui;
        let frame = if crate::tweaks::current().reduce_motion { 0 } else { (ui.t * 30.0) as u32 };
        let level = if frame == 0 { 1.0 } else { crate::title_crt::flicker(frame) };
        let reveal = holo::card_reveal(c.state, ui.t);
        let accent = if c.side == 0 { hex("#4db8ff") } else { hex("#ff6b8a") };
        ui.rect(x, y, w, h, Color::new(0.02, 0.05, 0.1, 0.55));
        let glow = match c.state {
            CardState::Pending => 0.25,
            CardState::Loading => 0.55 + 0.25 * (ui.t * 6.0).sin(),
            CardState::Ready => 0.8,
        };
        ui.rect_lines(x, y, w, h, 2.0, Color::new(accent.r, accent.g, accent.b, glow));
        if let Some((tex, pw, ph)) = &c.tex {
            let k = ((w - 12.0) / pw).min((h - 12.0) / ph);
            let (dw, dh) = (pw * k, ph * k);
            let (dx, dy) = (x + (w - dw) / 2.0, y + (h - dh) / 2.0);
            let rows = holo::STRIPS;
            let (sh, dsh) = (ph / rows as f32, dh / rows as f32);
            for s in 0..rows {
                let a = holo::strip_alpha(reveal, s);
                let a = if c.state == CardState::Pending { 0.1 } else { a };
                if a <= 0.0 {
                    continue;
                }
                let solid = if c.state == CardState::Ready { 1.0 } else { 0.0 };
                let tint = crate::poly::lerp_c(CYAN, WHITE, solid);
                let jit = if c.state == CardState::Loading { holo::jitter(s, frame) * 6.0 } else { 0.0 };
                draw_texture_ex(
                    tex,
                    ui.x(dx + jit),
                    ui.y(dy + s as f32 * dsh),
                    Color::new(tint.r * level, tint.g * level, tint.b * level, a * if solid > 0.0 { 1.0 } else { 0.7 }),
                    DrawTextureParams {
                        dest_size: Some(vec2(dw * ui.s, (dsh + 0.6) * ui.s)),
                        source: Some(Rect::new(0.0, s as f32 * sh, *pw, sh)),
                        ..Default::default()
                    },
                );
            }
            if c.state == CardState::Loading {
                let ey = dy + dh * holo::edge_fraction(reveal);
                ui.rect(x + 4.0, ey - 1.0, w - 8.0, 2.0, Color::new(0.8, 1.0, 1.0, 0.9));
                ui.gradient_v(x + 4.0, ey - 14.0, w - 8.0, 14.0, Color::new(0.35, 0.9, 1.0, 0.0), Color::new(0.35, 0.9, 1.0, 0.35));
            }
        }
        if c.tex.is_none() {
            let initial: String = c.name.chars().take(1).collect::<String>().to_uppercase();
            let a = if c.state == CardState::Pending { 0.25 } else { 0.7 };
            ui.text(&initial, x + w / 2.0, y + h * 0.62, (w * 0.55).max(12.0), Color::new(CYAN.r, CYAN.g, CYAN.b, a), Align::Center, true);
        }
        let mut sy = y;
        while sy < y + h {
            ui.rect(x, sy, w, 1.0, Color::new(0.0, 0.0, 0.03, 0.28));
            sy += 3.0;
        }
        let beam = (ui.t * 0.5 + c.side as f32 * 0.3).fract();
        ui.rect(x, y + beam * h, w, 6.0, Color::new(0.6, 0.95, 1.0, 0.07));
        if labels {
            let (word, col) = match c.state {
                CardState::Pending => (crate::i18n::tr3("NA FILA", "QUEUED", "EN COLA"), hex("#6f7a99")),
                CardState::Loading => (crate::i18n::tr3("CARREGANDO", "LOADING", "CARGANDO"), CYAN),
                CardState::Ready => (crate::i18n::tr3("PRONTO", "READY", "LISTO"), hex("#9dffb0")),
            };
            let name: String = c.name.chars().take(18).collect();
            ui.text(&name, x + w / 2.0, y + h + 24.0, 18.0, WHITE, Align::Center, true);
            ui.text(word, x + w / 2.0, y + h + 46.0, 14.0, col, Align::Center, true);
        }
    }
}
