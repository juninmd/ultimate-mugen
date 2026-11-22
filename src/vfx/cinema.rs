use super::Vfx;
use crate::gfx::*;
use crate::poly;
use crate::ui::Ui;
use ume_engine::matchs::Match;

const BAR_HEIGHT_MAX: f32 = 42.0;

#[derive(Clone, Copy, Default)]
pub struct CinemaDirector {
    pub bars: f32,
    pub bars_target: f32,
    pub bars_hold: f32,
    pub bars_col: [f32; 3],
    pub cutin_t: f32,
    pub cutin_owner: u32,
    pub cutin_col: [f32; 3],
    pub cutin_side: usize,
    pub dramatic: bool,
    pub focus_bias: f32,
    pub focus_x: f32,
    pub focus_y: f32,
}

impl CinemaDirector {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn trigger_cutin(&mut self, fighter: u32, side: usize, col: [f32; 3], dur: f32) {
        self.cutin_t = dur.max(self.cutin_t);
        self.cutin_owner = fighter;
        self.cutin_col = col;
        self.cutin_side = side;
        self.bars_hold = self.bars_hold.max(dur * 0.9);
        self.bars_col = col;
    }

    pub fn trigger_bars(&mut self, level: f32, hold: f32, col: [f32; 3]) {
        self.bars_target = self.bars_target.max(level.clamp(0.0, 1.0));
        self.bars_hold = self.bars_hold.max(hold);
        self.bars_col = col;
    }

    pub fn tick_match(&mut self, m: &Match) {
        let super_on = m.pause.as_ref().is_some_and(|p| p.is_super);
        let zoom_on = m.zoom_request.as_ref().is_some_and(|z| z.scale > 1.08);
        let throw_on = m
            .fighters
            .iter()
            .any(|f| !f.destroyed && (3800..3950).contains(&f.state_no));
        let slow_on = m.slow_mo > 0;
        if super_on || zoom_on || throw_on || slow_on {
            self.bars_hold = self.bars_hold.max(0.28);
            self.bars_target = self
                .bars_target
                .max(if super_on || throw_on { 1.0 } else { 0.72 });
        }
        if throw_on {
            if let Some((x, y)) = m
                .fighters
                .iter()
                .find(|f| !f.destroyed && (3800..3950).contains(&f.state_no))
                .map(|f| (f.pos.x, f.pos.y - 42.0))
            {
                self.focus_x = x;
                self.focus_y = y;
                self.focus_bias = (self.focus_bias + 0.16).min(0.65);
            }
        } else if self.focus_bias > 0.0 {
            self.focus_bias = (self.focus_bias - 0.08).max(0.0);
        }
    }

    pub fn update(&mut self, dt: f32, super_active: bool, ko_t: f32) {
        if self.cutin_t > 0.0 {
            self.cutin_t = (self.cutin_t - dt).max(0.0);
        }
        if self.bars_hold > 0.0 {
            self.bars_hold = (self.bars_hold - dt).max(0.0);
        }
        let target = if super_active || ko_t > 0.05 {
            1.0
        } else if self.bars_hold > 0.0 {
            self.bars_target.max(0.6)
        } else {
            self.dramatic = false;
            0.0
        };
        if target == 0.0 {
            self.bars_target = 0.0;
        }
        let rate = if target > self.bars { 7.5 } else { 3.2 };
        self.bars += (target - self.bars) * (rate * dt).min(1.0);
    }

    pub fn draw_bars(&self, ui: &Ui) {
        if self.bars <= 0.02 {
            return;
        }
        let e = self.bars * self.bars * (3.0 - 2.0 * self.bars);
        let h = BAR_HEIGHT_MAX * e;
        let col = Color::new(0.01, 0.01, 0.03, 0.94 * e.min(1.0));
        ui.rect(0.0, 0.0, 1280.0, h, col);
        ui.rect(0.0, 720.0 - h, 1280.0, h, col);
        let accent = if self.bars_col == [0.0, 0.0, 0.0] {
            [1.0, 0.78, 0.28]
        } else {
            self.bars_col
        };
        let line = Color::new(accent[0], accent[1], accent[2], 0.55 * e);
        let p_top = [
            vec2(ui.x(0.0), ui.y(h - 1.5)),
            vec2(ui.x(1280.0), ui.y(h - 1.5)),
            vec2(ui.x(1280.0), ui.y(h)),
            vec2(ui.x(0.0), ui.y(h)),
        ];
        let p_bot = [
            vec2(ui.x(0.0), ui.y(720.0 - h)),
            vec2(ui.x(1280.0), ui.y(720.0 - h)),
            vec2(ui.x(1280.0), ui.y(720.0 - h + 1.5)),
            vec2(ui.x(0.0), ui.y(720.0 - h + 1.5)),
        ];
        poly::solid(&p_top, line);
        poly::solid(&p_bot, line);
    }
}

impl Vfx {
    pub(super) fn on_cinema_event(
        &mut self,
        fighter: u32,
        bars: f32,
        slow: i32,
        invert: bool,
        lines: f32,
        flash: [u8; 3],
        m: &Match,
    ) {
        let f = m
            .fighters
            .iter()
            .find(|x| x.id == fighter || x.root == fighter);
        let col = f.map(Self::form_color).unwrap_or([1.0, 0.8, 0.3]);
        self.cinema
            .trigger_bars(bars, (slow as f32 / 60.0).max(0.55), col);
        if slow > 0 {
            self.slow_t = self.slow_t.max((slow as f32 / 60.0).clamp(0.25, 2.2));
        }
        if invert {
            self.force_impact();
        }
        if lines > 0.0 {
            self.lines = self.lines.max(lines.min(1.0));
            self.lines_col = col;
        }
        if flash != [0, 0, 0] {
            self.flash_add(0.45);
            self.chroma = self.chroma.max(0.5);
            self.radial = self.radial.max(0.45);
        }
    }

    pub(super) fn on_dramatic_ko(&mut self, loser_side: usize, m: &Match) {
        let winner = 1 - loser_side;
        let match_point = m.wins[winner] + 1 >= m.rounds_to_win;
        let super_recent = self.super_t > 0.08 || m.pause.as_ref().is_some_and(|p| p.is_super);
        if !(match_point || super_recent) {
            return;
        }
        self.cinema.dramatic = true;
        self.cinema.trigger_bars(1.0, 2.1, [1.0, 0.82, 0.28]);
        self.slow_t = self.slow_t.max(2.1);
        self.force_impact();
        self.lines = 1.0;
        self.lines_col = [1.0, 0.85, 0.3];
        self.radial = 0.85;
        self.chroma = 0.85;
        self.vig = 1.0;
        let title = if super_recent {
            crate::i18n::tr3(
                "FINALIZACAO DRAMATICA!",
                "DRAMATIC FINISH!",
                "FINAL DRAMATICO!",
            )
        } else {
            crate::i18n::tr3("GOLPE DECISIVO!", "CLIMAX FINISH!", "GOLPE DECISIVO!")
        };
        let winner_name = m
            .roots()
            .find(|f| f.side == winner)
            .map(|f| f.def.display_name.to_uppercase())
            .unwrap_or_default();
        self.banner(title, &winner_name, [1.0, 0.84, 0.25]);
    }
}
