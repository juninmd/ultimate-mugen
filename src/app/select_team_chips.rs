use super::select_mode::TEAM_MAX;
use super::*;

const RIGHT: f32 = 1240.0;
const MODE_Y: f32 = 7.0;
const SIZE_Y: f32 = 36.0;
const H: f32 = 27.0;
const STEP_W: f32 = 38.0;
const MODE_W: f32 = 178.0;
const SIZE_W: f32 = 84.0;
const GAP: f32 = 6.0;

impl App {
    fn team_chip(&self, x: f32, y: f32, w: f32, label: &str, on: bool) -> bool {
        let hover = self.hover_rect(x, y, w, H);
        let fill = if on {
            Color::new(1.0, 0.8, 0.2, 0.9)
        } else if hover {
            Color::new(1.0, 1.0, 1.0, 0.2)
        } else {
            Color::new(1.0, 1.0, 1.0, 0.08)
        };
        self.ui.rect(x, y, w, H, fill);
        let ink = if on {
            BLACK
        } else {
            Color::new(1.0, 1.0, 1.0, 0.85)
        };
        let text = self.fit_label(label, 13.0, w - 6.0);
        self.ui
            .text(&text, x + w / 2.0, y + 19.0, 13.0, ink, Align::Center, true);
        self.clicked_rect(x, y, w, H)
    }

    pub(crate) fn select_team_chips(&mut self) -> bool {
        if !matches!(self.screen, Screen::Select { .. })
            || !super::select_mode::in_family(self.mode)
        {
            return false;
        }
        let mut x = RIGHT - (STEP_W * 2.0 + MODE_W + GAP * 2.0);
        let prev = self.team_chip(x, MODE_Y, STEP_W, "<", false);
        x += STEP_W + GAP;
        let mode_click = self.team_chip(x, MODE_Y, MODE_W, self.mode.label(), self.mode.is_team());
        x += MODE_W + GAP;
        let next = self.team_chip(x, MODE_Y, STEP_W, ">", false);
        if prev || next || mode_click {
            return self.step_select_mode(if prev { -1 } else { 1 });
        }
        if !self.mode.is_team() {
            return false;
        }
        let mut x = RIGHT - (STEP_W * 2.0 + SIZE_W + GAP * 2.0);
        let minus = self.team_chip(x, SIZE_Y, STEP_W, "-", false);
        x += STEP_W + GAP;
        self.team_chip(x, SIZE_Y, SIZE_W, &format!("{0}v{0}", self.team_n), true);
        x += SIZE_W + GAP;
        let plus = self.team_chip(x, SIZE_Y, STEP_W, "+", self.team_n >= TEAM_MAX);
        if minus || plus {
            self.adjust_team_n(if minus { -1 } else { 1 });
            return true;
        }
        false
    }
}
