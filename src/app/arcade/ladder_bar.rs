use crate::app::{App, Mode};
use crate::gfx::*;
use crate::i18n::tr3;
use crate::ui::*;

pub(crate) struct ArcadeProgress {
    pub match_num: usize,
    pub total: usize,
    pub is_boss: bool,
}

pub(crate) fn arcade_progress(ladder: &[usize], idx: usize, boss: Option<usize>) -> Option<ArcadeProgress> {
    if ladder.is_empty() {
        return None;
    }
    let clamped = idx.min(ladder.len() - 1);
    let is_boss = boss.is_some_and(|b| ladder[clamped] == b);
    Some(ArcadeProgress {
        match_num: clamped + 1,
        total: ladder.len(),
        is_boss,
    })
}

impl App {
    pub(crate) fn draw_arcade_ladder_bar(&self, t: f32) {
        if self.mode != Mode::Arcade {
            return;
        }
        let Some(run) = self.arcade.as_ref() else {
            return;
        };
        let boss = self.boss_idx();
        let Some(prog) = arcade_progress(&run.ladder, run.index, boss) else {
            return;
        };
        let reveal = (t * 2.5).clamp(0.0, 1.0);
        let (x, y, w, h) = (180.0, 114.0, VW - 360.0, 34.0);
        crate::menu_art::glass(&self.ui, x, y, w, h, reveal);
        let accent = if prog.is_boss { hex("#ff4d6d") } else { hex("#ffcc33") };
        self.ui.rect(x, y, w * reveal, 2.0, accent);

        let count = run.ladder.len().min(11);
        let gap = 6.0;
        let inner_w = w - 20.0;
        let step_w = (inner_w - gap * (count.saturating_sub(1) as f32)) / count.max(1) as f32;
        let ny = y + 5.0;
        let nh = 24.0;
        for (i, &char_idx) in run.ladder.iter().take(count).enumerate() {
            let nx = x + 10.0 + i as f32 * (step_w + gap);
            let cleared = i < run.index;
            let active = i == run.index;
            let rung_boss = boss == Some(char_idx);
            let bg = if active {
                Color::new(accent.r, accent.g, accent.b, 0.92)
            } else if cleared {
                Color::new(0.18, 0.62, 0.36, 0.78)
            } else if rung_boss {
                Color::new(0.55, 0.14, 0.28, 0.62)
            } else {
                Color::new(1.0, 1.0, 1.0, 0.12)
            };
            self.ui.rect(nx, ny, step_w, nh, bg);
            let short_name = self
                .reg
                .chars
                .get(char_idx)
                .map(|c| c.name.split_whitespace().next().unwrap_or(&c.name))
                .unwrap_or("CPU")
                .to_uppercase();
            let label = if rung_boss && !active {
                tr3("CHEFE", "BOSS", "JEFE").to_string()
            } else {
                short_name.chars().take(7).collect()
            };
            let fg = if active { hex("#12051f") } else { WHITE };
            self.ui.text(&label, nx + step_w * 0.5, ny + 17.0, 11.0, fg, Align::Center, active || rung_boss);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arcade_progress_tracks_regular_and_boss_rungs() {
        assert!(arcade_progress(&[], 0, Some(9)).is_none());
        let ladder = vec![1, 2, 3, 9];
        let p0 = arcade_progress(&ladder, 0, Some(9)).unwrap();
        assert_eq!((p0.match_num, p0.total, p0.is_boss), (1, 4, false));
        let p3 = arcade_progress(&ladder, 3, Some(9)).unwrap();
        assert_eq!((p3.match_num, p3.total, p3.is_boss), (4, 4, true));
    }
}
